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

//! PathId: base64url path segments and their axum extractor.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use tower::ServiceExt;

use super::PathId;

/// A DID with a percent-encoded port survives the round trip unchanged.
#[test]
fn round_trips_a_did_with_an_encoded_port() {
    let did = "did:web:127.0.0.1%3A1500";
    let segment = PathId::encode(did);
    assert_eq!(segment, "ZGlkOndlYjoxMjcuMC4wLjElM0ExNTAw");
    assert_eq!(PathId::decode(&segment).unwrap(), did);
}

/// A segment that is not base64url is a bad format, not a panic.
#[test]
fn rejects_a_segment_that_is_not_base64url() {
    assert!(PathId::decode("did:web:host").is_err());
}

/// The extractor hands the handler the decoded id and answers 400 on a raw DID.
#[tokio::test]
async fn extractor_decodes_the_segment() {
    let app = Router::new().route("/minions/{id}", get(|PathId(id): PathId| async move { id }));
    let ok = app
        .clone()
        .oneshot(
            Request::get("/minions/ZGlkOndlYjoxMjcuMC4wLjElM0ExNTAw")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let body = axum::body::to_bytes(ok.into_body(), 1024).await.unwrap();
    assert_eq!(&body[..], b"did:web:127.0.0.1%3A1500");

    let bad = app
        .oneshot(
            Request::get("/minions/did:web:127.0.0.1%3A1500")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}
