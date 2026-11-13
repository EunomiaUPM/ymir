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

use serde_norway::from_str;
use ymir::config::traits::EventsConfigTrait;
use ymir::config::types::{
    CommonConfig, EventNodeIdentity, EventsConfig, KafkaFailureMode, KafkaKeyStrategy,
    KafkaSaslMechanism, KafkaSecurityProtocol, KafkaTopicStrategy,
};
use ymir::telemetry::{Telemetry, TraceParent};

#[test]
fn common_config_defaults_events_when_omitted() {
    let yaml = r#"
hosts:
  http:
    protocol: "http"
    url: "localhost"
    port: "8080"
    internal_port: "8080"
db:
  db_type: Postgres
  url: "localhost"
  port: "5432"
api:
  version: "v1"
  openapi_path: "/tmp/openapi.yaml"
connection:
  is_prod: false
  is_vault_real: false
  has_proxy: false
"#;
    let common: CommonConfig = from_str(yaml).expect("valid CommonConfig without events block");
    let events = common.events();
    assert!(!events.kafka.enabled);
    assert!(events.otlp.enabled);
    assert!(!events.stdout_json.enabled);
    assert!(!events.remote_bus.enabled);
    assert!(events.validate().is_ok());
}

#[test]
fn kafka_config_validates_when_enabled_and_redacts_secrets() {
    let yaml = r#"
dataspace_id: "ds-eunomia"
kafka:
  enabled: true
  brokers: "kafka-1:9092,kafka-2:9092"
  topic: "ds.audit"
  topic_strategy: by_category
  key_strategy: correlation_or_subject
  failure_mode: outbox_relay
  security_protocol: SASL_SSL
  sasl_mechanism: SCRAM-SHA-256
  sasl_username: "agent"
  sasl_password: "super-secret-password"
  ssl_key_password: "key-secret-password"
"#;
    let cfg: EventsConfig = from_str(yaml).expect("valid Kafka EventsConfig");
    assert!(cfg.validate().is_ok());
    assert_eq!(cfg.kafka.topic_strategy, KafkaTopicStrategy::ByCategory);
    assert_eq!(cfg.kafka.key_strategy, KafkaKeyStrategy::CorrelationOrSubject);
    assert_eq!(cfg.kafka.failure_mode, KafkaFailureMode::OutboxRelay);
    assert_eq!(cfg.kafka.security_protocol, KafkaSecurityProtocol::SaslSsl);
    assert_eq!(cfg.kafka.sasl_mechanism, Some(KafkaSaslMechanism::ScramSha256));

    let debug_str = format!("{:?}", cfg.kafka);
    assert!(!debug_str.contains("super-secret-password"));
    assert!(!debug_str.contains("key-secret-password"));
    assert!(debug_str.contains("[REDACTED]"));
}

#[test]
fn kafka_enabled_without_brokers_fails_validation() {
    let mut cfg = EventsConfig::default();
    cfg.kafka.enabled = true;
    assert!(cfg.validate().is_err());
}

#[test]
fn remote_bus_requires_http_url_when_enabled() {
    let mut cfg = EventsConfig::default();
    cfg.remote_bus.enabled = true;
    assert!(cfg.validate().is_err());

    cfg.remote_bus.endpoint = Some("ftp://invalid".to_string());
    assert!(cfg.validate().is_err());

    cfg.remote_bus.endpoint = Some("https://node.local/api/v1/events".to_string());
    assert!(cfg.validate().is_ok());
}

#[test]
fn node_identity_tracks_service_and_late_bound_participant() {
    let cfg = EventsConfig {
        dataspace_id: Some("ds-1".to_string()),
        ..Default::default()
    };
    let identity = EventNodeIdentity::from_config(&cfg, "monolith");
    identity.set_default_service("monolith");
    identity.set_participant_id(Some("did:web:provider.example".to_string()));
    let snap = identity.snapshot();
    assert_eq!(snap.dataspace_id.as_deref(), Some("ds-1"));
    assert_eq!(snap.service, "monolith");
    assert_eq!(snap.participant_id.as_deref(), Some("did:web:provider.example"));

    let mut pinned = EventsConfig::default();
    pinned.service_name = Some("custom-service".to_string());
    let pinned_id = EventNodeIdentity::from_config(&pinned, "fallback");
    pinned_id.set_default_service("ignored-override");
    assert_eq!(pinned_id.snapshot().service, "custom-service");
}

#[test]
fn traceparent_parser_extracts_valid_span_context() {
    let parsed = TraceParent::parse_context(
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
    )
    .expect("valid traceparent");
    assert_eq!(
        parsed.trace_id().to_string(),
        "4bf92f3577b34da6a3ce929d0e0e4736"
    );
    assert_eq!(parsed.span_id().to_string(), "00f067aa0ba902b7");
    assert!(parsed.is_sampled());
    assert!(TraceParent::parse_context("not-a-traceparent").is_none());
    assert!(Telemetry::logger_provider().is_none());
}
