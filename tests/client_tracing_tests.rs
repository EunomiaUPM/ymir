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

//! Outbound calls carry the caller's W3C trace context.

use axum::Router;
use axum::http::HeaderMap;
use axum::routing::get;
use opentelemetry::global;
use opentelemetry::trace::{SpanKind, TraceContextExt, TracerProvider as _};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider};
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::layer::SubscriberExt;
use ymir::services::client::{ClientService, ClientTrait};

#[tokio::test]
async fn client_span_is_child_of_caller_and_propagated() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let exporter = InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let subscriber = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test")));
    let _guard = tracing::subscriber::set_default(subscriber);

    let app = Router::new().route(
        "/",
        get(|headers: HeaderMap| async move {
            headers
                .get("traceparent")
                .map(|v| v.to_str().unwrap().to_owned())
                .unwrap_or_default()
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let caller = tracing::info_span!("caller");
    let trace_id = caller.context().span().span_context().trace_id();
    let client = ClientService::builder().build();
    let traceparent = async { client.get(&url, None).await.unwrap().text().await.unwrap() }
        .instrument(caller.clone())
        .await;
    drop(caller);

    assert!(
        traceparent.starts_with(&format!("00-{trace_id}-")),
        "traceparent {traceparent:?} does not continue trace {trace_id}"
    );
    provider.force_flush().unwrap();
    let spans = exporter.get_finished_spans().unwrap();
    let client_span = spans
        .iter()
        .find(|s| s.span_kind == SpanKind::Client)
        .expect("client span exported");
    assert_eq!(client_span.span_context.trace_id(), trace_id);
    // The server sees the client span, not the caller, as its parent.
    let sent_parent = traceparent.split('-').nth(2).unwrap();
    assert_eq!(client_span.span_context.span_id().to_string(), sent_parent);
}
