/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Per-host breaker, bulkhead, retry budget and trace propagation of `ClientService`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use axum::Router;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use ymir::errors::{Errors, PetitionFailure};
use ymir::services::client::{ClientService, ClientTrait};

async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Answers `status` and counts how many requests actually reached it.
async fn counting_server(status: StatusCode) -> (String, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    let app = Router::new().route(
        "/",
        get(move || {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                status
            }
        }),
    );
    (serve(app).await, hits)
}

fn failure(err: &Errors) -> &PetitionFailure {
    match err {
        Errors::PetitionError { failure, .. } => failure,
        other => panic!("unexpected error {other:?}"),
    }
}

#[tokio::test]
async fn breaker_opens_after_consecutive_failures_and_fails_fast() {
    let (url, hits) = counting_server(StatusCode::INTERNAL_SERVER_ERROR).await;
    let client = ClientService::builder()
        .circuit_breaker(3, Duration::from_secs(60))
        .build();

    for _ in 0..3 {
        let err = client.get(&url, None).await.unwrap_err();
        assert!(matches!(failure(&err), PetitionFailure::HttpStatus(_)));
    }
    let err = client.get(&url, None).await.unwrap_err();
    assert!(matches!(failure(&err), PetitionFailure::CircuitOpen));
    assert_eq!(
        hits.load(Ordering::SeqCst),
        3,
        "an open circuit must not reach the host"
    );
    assert_eq!(client.degraded_hosts().len(), 1);
}

#[tokio::test]
async fn client_errors_do_not_open_the_breaker() {
    let (url, hits) = counting_server(StatusCode::NOT_FOUND).await;
    let client = ClientService::builder()
        .circuit_breaker(2, Duration::from_secs(60))
        .build();

    for _ in 0..5 {
        let res = client.get(&url, None).await.unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
    assert_eq!(hits.load(Ordering::SeqCst), 5);
    assert!(client.degraded_hosts().is_empty());
}

#[tokio::test]
async fn half_open_probe_closes_the_breaker_once_the_host_recovers() {
    let healthy = Arc::new(AtomicBool::new(false));
    let flag = healthy.clone();
    let app = Router::new().route(
        "/",
        get(move || {
            let flag = flag.clone();
            async move {
                match flag.load(Ordering::SeqCst) {
                    true => StatusCode::OK,
                    false => StatusCode::SERVICE_UNAVAILABLE,
                }
            }
        }),
    );
    let url = serve(app).await;
    let client = ClientService::builder()
        .circuit_breaker(1, Duration::from_millis(100))
        .build();

    assert!(client.get(&url, None).await.is_err());
    let err = client.get(&url, None).await.unwrap_err();
    assert!(matches!(failure(&err), PetitionFailure::CircuitOpen));

    healthy.store(true, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(
        client.get(&url, None).await.unwrap().status(),
        StatusCode::OK
    );
    assert!(client.degraded_hosts().is_empty());
}

#[tokio::test]
async fn a_saturated_host_does_not_block_other_hosts() {
    let slow = Router::new().route(
        "/",
        get(|| async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            StatusCode::OK
        }),
    );
    let slow_url = serve(slow).await;
    let (fast_url, _) = counting_server(StatusCode::OK).await;
    let client = Arc::new(
        ClientService::builder()
            .concurrency(2)
            .per_host_concurrency(1)
            .build(),
    );

    // Two calls to the slow host: one in flight, one queued on its bulkhead.
    for _ in 0..2 {
        let (client, url) = (client.clone(), slow_url.clone());
        tokio::spawn(async move { client.get(&url, None).await });
    }
    tokio::time::sleep(Duration::from_millis(100)).await;

    let started = Instant::now();
    let res = client.get(&fast_url, None).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[tokio::test]
async fn retry_after_beyond_the_deadline_gives_up_at_once() {
    let app = Router::new().route(
        "/",
        get(|| async {
            let mut headers = HeaderMap::new();
            headers.insert("retry-after", "5".parse().unwrap());
            (StatusCode::SERVICE_UNAVAILABLE, headers).into_response()
        }),
    );
    let url = serve(app).await;
    let client = ClientService::builder()
        .max_retries(3)
        .deadline(Duration::from_secs(1))
        .build();

    let started = Instant::now();
    let err = client.get(&url, None).await.unwrap_err();
    assert!(matches!(
        failure(&err),
        PetitionFailure::HttpStatus(StatusCode::SERVICE_UNAVAILABLE)
    ));
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[tokio::test]
async fn transient_failures_are_retried_within_budget() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let app = Router::new().route(
        "/",
        get(move || {
            let counter = counter.clone();
            async move {
                match counter.fetch_add(1, Ordering::SeqCst) {
                    0 => StatusCode::BAD_GATEWAY,
                    _ => StatusCode::OK,
                }
            }
        }),
    );
    let url = serve(app).await;
    let client = ClientService::builder()
        .max_retries(1)
        .retry_backoff(Duration::from_millis(10), Duration::from_millis(50))
        .build();

    assert_eq!(
        client.get(&url, None).await.unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
