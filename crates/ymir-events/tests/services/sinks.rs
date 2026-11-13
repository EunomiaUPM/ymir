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

//! Sink pipeline tests: OTLP logs, Kafka routing & mock outbox relay, remote bus HTTP relay,
//! structured tracing sink, and `CompositeEventSink` isolation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use chrono::Utc;
use opentelemetry::logs::AnyValue;
use opentelemetry::{SpanId, TraceId};
use opentelemetry_sdk::logs::{InMemoryLogExporter, SdkLoggerProvider};
use tokio::net::TcpListener;
use ymir::config::types::{
    EventNodeIdentity, EventSeverityLevel, EventsConfig, KafkaEventsConfig, KafkaFailureMode,
    KafkaKeyStrategy, KafkaTopicStrategy, OtlpEventsConfig, RemoteBusEventsConfig,
    StdoutEventsConfig,
};
use ymir::errors::Errors;
use ymir::oauth::Owner;
use ymir::types::participants::Visibility;
use ymir_events::data::repo::MockEventOutboxRepo;
use ymir_events::entities::commands::PublishEventRequest;
use ymir_events::entities::outbox::{OutboxRecord, OutboxStatus};
use ymir_events::schemas::dataspace::DataAccessPayload;
use ymir_events::schemas::identity::{GrantAction, GrantDirection, IdentityGrantPayload};
use ymir_events::{
    CompositeEventSink, DeliveryClass, EventCategory, EventContext, EventEnvelope, EventPayload,
    EventPipelineBuilder, EventSeverity, EventSinkTrait, EventSource, KafkaRouting,
    MockEventSinkTrait, MockKafkaProducerPort, OtlpEventSink, RemoteBusSink, TracingJsonSink,
    TypedEventSink,
};

#[cfg(feature = "kafka")]
use ymir_events::{KafkaEventSink, KafkaOutboxWorker};
#[cfg(not(feature = "kafka"))]
use ymir_events::services::event_sink::kafka::{KafkaEventSink, KafkaOutboxWorker};

fn test_owner() -> Owner {
    Owner::new("user-1", "/admin/upm".parse().unwrap(), Visibility::Public)
}

fn node_identity() -> Arc<EventNodeIdentity> {
    let cfg = EventsConfig {
        dataspace_id: Some("ds-demo".to_string()),
        service_name: Some("control-plane".to_string()),
        instance_id: Some("cp-01".to_string()),
        ..Default::default()
    };
    let id = Arc::new(EventNodeIdentity::from_config(&cfg, "control-plane"));
    id.set_participant_id(Some("did:web:alice.example".to_string()));
    id
}

fn sample_data_access(transfer_id: &str, dataset_id: Option<&str>, bytes: Option<u64>) -> DataAccessPayload {
    DataAccessPayload {
        allowed: true,
        transfer_id: transfer_id.to_string(),
        agreement_id: Some("ag-1".to_string()),
        dataset_id: dataset_id.map(ToOwned::to_owned),
        channel: "http_proxy".to_string(),
        method: Some("GET".to_string()),
        path: Some("/".to_string()),
        status_code: Some(200),
        bytes_transferred: bytes,
        duration_ms: None,
        peer_id: None,
        denial_reason: None,
    }
}

fn sample_denied_grant(grant_id: &str, reason: &str) -> IdentityGrantPayload {
    IdentityGrantPayload {
        action: GrantAction::Rejected,
        grant_id: grant_id.to_string(),
        direction: GrantDirection::Received,
        kind: "consumer".to_string(),
        status: "Rejected".to_string(),
        previous_status: None,
        peer_participant_id: None,
        peer_participant_nick: None,
        vc_types: vec![],
        token_fingerprint: None,
        expires_at: None,
        reason: Some(reason.to_string()),
    }
}

fn attr_string(record: &opentelemetry_sdk::logs::SdkLogRecord, key: &str) -> Option<String> {
    record.attributes_iter().find_map(|(k, v)| {
        if k.as_str() == key {
            match v {
                AnyValue::String(s) => Some(s.as_str().to_string()),
                _ => None,
            }
        } else {
            None
        }
    })
}

#[tokio::test]
async fn otlp_sink_exports_canonical_attributes_severity_and_trace_context() {
    let exporter = InMemoryLogExporter::default();
    let provider = SdkLoggerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();

    let sink = OtlpEventSink::new(
        provider.clone(),
        OtlpEventsConfig {
            enabled: true,
            min_severity: EventSeverityLevel::Warn,
            include_full_payload: true,
        },
    );

    // Info event is filtered out by min_severity = Warn.
    let info_ctx = EventContext::for_owner(test_owner());
    let info_payload = sample_data_access("tr-info", None, Some(128));
    sink.emit(&info_ctx, &info_payload).await.unwrap();

    // Warn event with explicit traceparent is exported and linked.
    let trace_id_hex = "4bf92f3577b34da6a3ce929d0e0e4736";
    let span_id_hex = "00f067aa0ba902b7";
    let warn_ctx = EventContext::for_owner(test_owner())
        .with_transfer_id("tr-9")
        .with_trace_context(Some(format!("00-{trace_id_hex}-{span_id_hex}-01")));
    let source = EventSource::from_snapshot(&node_identity().snapshot(), "heimdall");
    let denied = sample_denied_grant("grant-7", "expired")
        .into_envelope(&warn_ctx, source)
        .unwrap();

    sink.emit_envelope(denied.clone()).await.unwrap();
    provider.force_flush().unwrap();

    let logs = exporter.get_emitted_logs().unwrap();
    assert_eq!(logs.len(), 1);
    let record = &logs[0].record;
    assert_eq!(record.event_name(), Some("dataspace.event"));
    assert_eq!(record.severity_text(), Some("WARN"));
    assert_eq!(attr_string(record, "dataspace.id").as_deref(), Some("ds-demo"));
    assert_eq!(
        attr_string(record, "participant.id").as_deref(),
        Some("did:web:alice.example")
    );
    assert_eq!(
        attr_string(record, "event.topic").as_deref(),
        Some("grants:rejected")
    );
    assert_eq!(
        attr_string(record, "event.category").as_deref(),
        Some("AuditLifecycle")
    );
    assert_eq!(
        attr_string(record, "event.outcome").as_deref(),
        Some("Denied")
    );
    assert_eq!(
        attr_string(record, "flow.grant_id").as_deref(),
        Some("grant-7")
    );
    assert!(attr_string(record, "event.payload").is_some());

    let tc = record.trace_context().expect("trace context attached");
    assert_eq!(tc.trace_id, TraceId::from_hex(trace_id_hex).unwrap());
    assert_eq!(tc.span_id, SpanId::from_hex(span_id_hex).unwrap());
}

#[tokio::test]
async fn stdout_json_sink_handles_threshold_safely() {
    let sink = TracingJsonSink::new(StdoutEventsConfig {
        enabled: true,
        min_severity: EventSeverityLevel::Info,
    });
    let payload = sample_data_access("tr-1", None, Some(10));
    sink.emit(&EventContext::for_owner(test_owner()), &payload)
        .await
        .unwrap();
}

#[tokio::test]
async fn remote_bus_sink_preserves_identity_headers_and_correlation() {
    type Captured = Arc<Mutex<Vec<(HeaderMap, PublishEventRequest)>>>;
    let captured: Captured = Arc::default();

    let app = Router::new()
        .route(
            "/api/v1/events",
            post(
                |State(store): State<Captured>,
                 headers: HeaderMap,
                 Json(body): Json<PublishEventRequest>| async move {
                    store.lock().unwrap().push((headers, body));
                    StatusCode::CREATED
                },
            ),
        )
        .with_state(captured.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let sink = RemoteBusSink::new(RemoteBusEventsConfig {
        enabled: true,
        endpoint: Some(format!("http://127.0.0.1:{port}/api/v1/events")),
        bearer_token: Some("relay-token-123".to_string()),
        timeout_ms: 2000,
    })
    .unwrap();

    let ctx = EventContext::for_owner(test_owner()).with_transfer_id("tr-55");
    let source = EventSource::from_snapshot(&node_identity().snapshot(), "dataplane");
    let env = sample_data_access("tr-55", Some("ds-1"), Some(500))
        .into_envelope(&ctx, source)
        .unwrap();

    sink.emit_envelope(env.clone()).await.unwrap();

    let lock = captured.lock().unwrap();
    assert_eq!(lock.len(), 1);
    let (headers, body) = &lock[0];
    assert_eq!(headers["authorization"], "Bearer relay-token-123");
    assert_eq!(body.id.as_ref(), Some(&env.id));
    assert_eq!(body.topic, "dataplane:data:accessed");
    assert_eq!(
        body.source.as_ref().unwrap().service,
        "control-plane"
    );
    assert_eq!(
        body.flow_refs.as_ref().unwrap().transfer_id.as_deref(),
        Some("tr-55")
    );
}

#[test]
fn kafka_routing_supports_single_domain_and_category_strategies() {
    let source = EventSource::from_snapshot(&node_identity().snapshot(), "transfer-agent");
    let ctx = EventContext::for_owner(test_owner()).with_correlation("corr-abc");
    let mut env = sample_data_access("tr-1", None, None)
        .into_envelope(&ctx, source)
        .unwrap();
    env.topic = "transfers:started".parse().unwrap();
    env.event_type = EventCategory::UsageControl;

    let mut cfg = KafkaEventsConfig {
        topic: "dataspace.events".to_string(),
        topic_prefix: "ds.v2".to_string(),
        topic_strategy: KafkaTopicStrategy::Single,
        key_strategy: KafkaKeyStrategy::CorrelationOrSubject,
        ..Default::default()
    };
    assert_eq!(
        KafkaRouting::resolve_topic(&cfg, &env),
        "dataspace.events"
    );
    assert_eq!(KafkaRouting::resolve_key(&cfg, &env), "corr-abc");

    cfg.topic_strategy = KafkaTopicStrategy::ByDomain;
    cfg.key_strategy = KafkaKeyStrategy::ParticipantId;
    assert_eq!(KafkaRouting::resolve_topic(&cfg, &env), "ds.v2.transfers");
    assert_eq!(
        KafkaRouting::resolve_key(&cfg, &env),
        "did:web:alice.example"
    );

    cfg.topic_strategy = KafkaTopicStrategy::ByCategory;
    cfg.key_strategy = KafkaKeyStrategy::EventId;
    assert_eq!(
        KafkaRouting::resolve_topic(&cfg, &env),
        "ds.v2.usage-control"
    );
    assert_eq!(KafkaRouting::resolve_key(&cfg, &env), env.id.to_string());

    let headers: HashMap<String, String> = KafkaRouting::build_headers(&env).into_iter().collect();
    assert_eq!(headers["spec_version"], "2.0");
    assert_eq!(headers["event_type"], "UsageControl");
    assert_eq!(headers["participant_id"], "did:web:alice.example");
}

#[tokio::test]
async fn kafka_sink_spools_only_audit_durable_events_when_outbox_relay_is_enabled() {
    let mut producer = MockKafkaProducerPort::new();
    producer
        .expect_produce()
        .times(2)
        .returning(|_| Err("broker unreachable".to_string()));

    let mut outbox = MockEventOutboxRepo::new();
    outbox
        .expect_enqueue()
        .withf(|rec| {
            rec.sink_name == "kafka"
                && rec.destination == "dataspace.events"
                && rec.status == OutboxStatus::Pending
                && rec.attempts == 1
        })
        .times(1)
        .returning(|_| Ok(()));

    let sink = KafkaEventSink::with_producer(
        KafkaEventsConfig {
            enabled: true,
            brokers: "localhost:9092".to_string(),
            failure_mode: KafkaFailureMode::OutboxRelay,
            ..Default::default()
        },
        Arc::new(producer),
        Some(Arc::new(outbox)),
    );

    let ctx = EventContext::for_owner(test_owner());
    let durable = sample_denied_grant("grant-1", "bad")
        .into_envelope(&ctx, EventSource::for_crate("heimdall"))
        .unwrap();
    assert_eq!(durable.delivery_class, DeliveryClass::AuditDurable);
    assert!(sink.deliver_envelope(durable).await.is_err());

    let telemetry = sample_data_access("tr-1", None, Some(1))
        .into_envelope(&ctx, EventSource::for_crate("dataplane"))
        .unwrap();
    assert_eq!(telemetry.delivery_class, DeliveryClass::TelemetryStream);
    // TelemetryStream is high-volume best-effort and must not bloat the durable outbox.
    assert!(sink.deliver_envelope(telemetry).await.is_err());
}

#[tokio::test]
async fn kafka_outbox_worker_handles_delivery_reschedule_and_dead_letter() {
    let now = Utc::now();
    let sample = |id: &str, attempts: u32| OutboxRecord {
        id: id.to_string(),
        sink_name: "kafka".to_string(),
        event_id: "urn:uuid:11111111-1111-1111-1111-111111111111".to_string(),
        destination: "dataspace.events".to_string(),
        partition_key: "key-1".to_string(),
        envelope_json: serde_json::json!({ "hello": "world" }),
        attempts,
        next_retry_at: now,
        last_error: Some("prev".to_string()),
        status: OutboxStatus::Pending,
        created_at: now,
        delivered_at: None,
    };

    let mut producer = MockKafkaProducerPort::new();
    let call_count = Arc::new(Mutex::new(0u32));
    let counter = call_count.clone();
    producer.expect_produce().times(3).returning(move |_| {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
        if *guard == 1 {
            Ok(())
        } else {
            Err("down".to_string())
        }
    });

    let mut outbox = MockEventOutboxRepo::new();
    let rec1 = sample("ob-ok", 0);
    let rec2 = sample("ob-retry", 0);
    let rec3 = sample("ob-dlq", 2);
    outbox
        .expect_get_due_pending()
        .times(1)
        .returning(move |_, _| Ok(vec![rec1.clone(), rec2.clone(), rec3.clone()]));
    outbox
        .expect_mark_delivered()
        .withf(|id, attempts| id == "ob-ok" && *attempts == 1)
        .times(1)
        .returning(|_, _| Ok(()));
    outbox
        .expect_reschedule()
        .withf(|id, attempts, _, _| id == "ob-retry" && *attempts == 1)
        .times(1)
        .returning(|_, _, _, _| Ok(()));
    outbox
        .expect_mark_dead_letter()
        .withf(|id, attempts, _| id == "ob-dlq" && *attempts == 3)
        .times(1)
        .returning(|_, _, _| Ok(()));

    let worker = KafkaOutboxWorker::with_producer(
        Arc::new(producer),
        Arc::new(outbox),
        Duration::from_secs(5),
        3,
    );

    assert_eq!(worker.poll_once().await.unwrap(), 1);
}

#[tokio::test]
async fn composite_sink_isolates_non_critical_failures_and_propagates_critical_error() {
    let mut local = MockEventSinkTrait::new();
    local
        .expect_emit_envelope()
        .withf(|env| {
            env.source.dataspace_id.as_deref() == Some("ds-demo")
                && env.source.participant_id.as_deref() == Some("did:web:alice.example")
        })
        .times(1)
        .returning(|_| Err(Errors::crazy("pg failed", None)));

    let mut failing_aux = MockEventSinkTrait::new();
    failing_aux
        .expect_emit_envelope()
        .times(1)
        .returning(|_| Err(Errors::crazy("aux down", None)));

    let mut healthy_aux = MockEventSinkTrait::new();
    healthy_aux
        .expect_emit_envelope()
        .times(1)
        .returning(|_| Ok(()));

    let mut composite = CompositeEventSink::new(node_identity());
    composite.add_critical_sink("local_store", Arc::new(local));
    composite.add_sink("otlp", Arc::new(failing_aux));
    composite.add_sink("kafka", Arc::new(healthy_aux));

    let ctx = EventContext::for_owner(test_owner());
    let env = sample_data_access("tr-1", None, None)
        .into_envelope(&ctx, EventSource::for_crate("dataplane"))
        .unwrap();

    // Critical local store failure still surfaces while healthy aux sink ran.
    assert!(composite.emit_envelope(env).await.is_err());
}

#[test]
fn pipeline_builder_constructs_with_kafka_disabled_and_validates_kafka_feature() {
    let mut mock_bus = MockEventSinkTrait::new();
    mock_bus.expect_emit_envelope().returning(|_| Ok(()));

    let default_cfg = EventsConfig::default();
    let built = EventPipelineBuilder::build(
        &default_cfg,
        node_identity(),
        Some(Arc::new(mock_bus)),
        None,
    )
    .unwrap();
    assert!(built.workers.is_empty());

    let kafka_cfg = EventsConfig {
        kafka: KafkaEventsConfig {
            enabled: true,
            brokers: "127.0.0.1:9092".to_string(),
            failure_mode: KafkaFailureMode::OutboxRelay,
            ..Default::default()
        },
        ..Default::default()
    };

    let outbox = Arc::new(MockEventOutboxRepo::new());
    let kafka_result = EventPipelineBuilder::build(&kafka_cfg, node_identity(), None, Some(outbox));

    #[cfg(feature = "kafka")]
    {
        let pipeline = kafka_result.unwrap();
        assert_eq!(pipeline.workers.len(), 1);
    }
    #[cfg(not(feature = "kafka"))]
    {
        assert!(kafka_result.is_err());
    }
}

#[test]
fn event_severity_orders_cleanly() {
    assert!(EventSeverity::Debug < EventSeverity::Info);
    assert!(EventSeverity::Info < EventSeverity::Warn);
    assert!(EventSeverity::Warn < EventSeverity::Error);
    assert!(EventSeverity::Error < EventSeverity::Critical);
}
