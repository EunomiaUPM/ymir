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

//! Event emission and multi-sink delivery configuration (`events` block in `CommonConfig`).

use std::fmt;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::errors::{Errors, Outcome};

/// Top-level event emission configuration shared by every Ymir agent binary.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct EventsConfig {
    /// Logical dataspace identifier stamped on every emitted event source.
    #[serde(default)]
    pub dataspace_id: Option<String>,
    /// Optional service name override in `event.source.service`.
    #[serde(default)]
    pub service_name: Option<String>,
    /// Optional instance identifier override in `event.source.instance_id`.
    #[serde(default)]
    pub instance_id: Option<String>,
    /// Local Postgres + SSE/WS + webhook event bus settings.
    #[serde(default)]
    pub local_store: LocalStoreEventsConfig,
    /// OpenTelemetry Logs (`SdkLoggerProvider`) event delivery settings.
    #[serde(default)]
    pub otlp: OtlpEventsConfig,
    /// Optional Apache Kafka backbone delivery settings (disabled by default).
    #[serde(default)]
    pub kafka: KafkaEventsConfig,
    /// Optional structured tracing/stdout JSON sink settings.
    #[serde(default)]
    pub stdout_json: StdoutEventsConfig,
    /// Optional HTTP relay to a central node event bus (for standalone workers).
    #[serde(default)]
    pub remote_bus: RemoteBusEventsConfig,
}

impl EventsConfig {
    fn non_empty(value: Option<&str>) -> Option<String> {
        value
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
    }

    fn non_empty_env(keys: &[&str]) -> Option<String> {
        keys.iter()
            .find_map(|key| std::env::var(key).ok().and_then(|v| Self::non_empty(Some(&v))))
    }

    /// Resolves the dataspace identifier from config or standard environment variables.
    pub fn resolve_dataspace_id(&self) -> Option<String> {
        Self::non_empty(self.dataspace_id.as_deref())
            .or_else(|| Self::non_empty_env(&["YMIR_DATASPACE_ID", "DATASPACE_ID"]))
    }

    /// Resolves the logical service name with fallback to environment and process default.
    pub fn resolve_service_name(&self, default_service: &str) -> String {
        Self::non_empty(self.service_name.as_deref())
            .or_else(|| Self::non_empty_env(&["YMIR_SERVICE_NAME", "OTEL_SERVICE_NAME"]))
            .or_else(|| Self::non_empty(Some(default_service)))
            .unwrap_or_else(|| "ymir-agent".to_string())
    }

    /// Resolves the instance identifier from config, host environment, or a fallback ID.
    pub fn resolve_instance_id(&self, fallback: &str) -> String {
        Self::non_empty(self.instance_id.as_deref())
            .or_else(|| Self::non_empty_env(&["YMIR_INSTANCE_ID", "HOSTNAME"]))
            .unwrap_or_else(|| fallback.to_string())
    }

    /// Validates enabled sink sub-configurations before constructing the event pipeline.
    pub fn validate(&self) -> Outcome<()> {
        self.kafka.validate()?;
        self.remote_bus.validate()?;
        Ok(())
    }
}

/// Controls whether events are persisted and fanned out by the local `EventBus`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct LocalStoreEventsConfig {
    /// Explicit override; `None` lets the boot decide (`true` when `EventsModule` is active).
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Optional retention window in days for high-frequency `TelemetryStream` rows.
    #[serde(default)]
    pub retention_telemetry_days: Option<u32>,
}

impl LocalStoreEventsConfig {
    /// Resolves whether the local Postgres/webhook bus is active for the process.
    pub fn should_enable(&self, has_local_bus: bool) -> bool {
        self.enabled.unwrap_or(has_local_bus)
    }
}

/// Canonical severity threshold shared by config and sink filters.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventSeverityLevel {
    #[serde(alias = "Debug", alias = "DEBUG")]
    #[default]
    Debug,
    #[serde(alias = "Info", alias = "INFO")]
    Info,
    #[serde(alias = "Warn", alias = "WARN")]
    Warn,
    #[serde(alias = "Error", alias = "ERROR")]
    Error,
    #[serde(alias = "Critical", alias = "CRITICAL")]
    Critical,
}

impl EventSeverityLevel {
    /// Stable lowercase severity label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

/// Configuration for exporting canonical events as OpenTelemetry `LogRecord`s.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OtlpEventsConfig {
    /// Enables OTLP emission whenever an OTel `SdkLoggerProvider` is initialized.
    #[serde(default = "OtlpEventsConfig::default_enabled")]
    pub enabled: bool,
    /// Minimum event severity forwarded to OTLP.
    #[serde(default)]
    pub min_severity: EventSeverityLevel,
    /// When true, attaches the full serialized payload JSON alongside indexed attributes.
    #[serde(default = "OtlpEventsConfig::default_include_full_payload")]
    pub include_full_payload: bool,
}

impl OtlpEventsConfig {
    fn default_enabled() -> bool {
        true
    }

    fn default_include_full_payload() -> bool {
        true
    }
}

impl Default for OtlpEventsConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            min_severity: EventSeverityLevel::Debug,
            include_full_payload: Self::default_include_full_payload(),
        }
    }
}

/// Kafka destination topic mapping strategy.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KafkaTopicStrategy {
    /// Routes all events to `kafka.topic` for unified single-stream log ingestion.
    #[serde(alias = "Single")]
    #[default]
    Single,
    /// Routes to `<topic_prefix>.<topic-first-segment>`.
    #[serde(alias = "ByDomain")]
    ByDomain,
    /// Routes to `<topic_prefix>.<kebab-case-event-category>`.
    #[serde(alias = "ByCategory")]
    ByCategory,
}

/// Kafka partition key selection strategy.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KafkaKeyStrategy {
    /// Uses `correlation_id`, then `flow_refs`, then `subject.id`, then `event.id`.
    #[serde(alias = "CorrelationOrSubject")]
    #[default]
    CorrelationOrSubject,
    /// Partitions strictly by emitting `source.participant_id` (or `event.id` if unset).
    #[serde(alias = "ParticipantId")]
    ParticipantId,
    /// Partitions uniformly by unique `event.id`.
    #[serde(alias = "EventId")]
    EventId,
}

/// Behavior when Kafka delivery fails or the bounded queue saturates.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KafkaFailureMode {
    /// Never blocks callers; records error metrics and tracing warnings.
    #[serde(alias = "BestEffortWithMetrics")]
    #[default]
    BestEffortWithMetrics,
    /// Spools failed durable events to the Postgres outbox when `local_store` is active.
    #[serde(alias = "OutboxRelay")]
    OutboxRelay,
}

/// Kafka wire security protocol (`rdkafka` `security.protocol`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KafkaSecurityProtocol {
    #[serde(rename = "PLAINTEXT", alias = "plaintext", alias = "Plaintext")]
    #[default]
    Plaintext,
    #[serde(rename = "SSL", alias = "ssl", alias = "Ssl")]
    Ssl,
    #[serde(rename = "SASL_PLAINTEXT", alias = "sasl_plaintext", alias = "SaslPlaintext")]
    SaslPlaintext,
    #[serde(rename = "SASL_SSL", alias = "sasl_ssl", alias = "SaslSsl")]
    SaslSsl,
}

impl KafkaSecurityProtocol {
    /// Returns the exact librdkafka `security.protocol` property value.
    pub fn as_rdkafka_str(&self) -> &'static str {
        match self {
            Self::Plaintext => "PLAINTEXT",
            Self::Ssl => "SSL",
            Self::SaslPlaintext => "SASL_PLAINTEXT",
            Self::SaslSsl => "SASL_SSL",
        }
    }
}

/// Kafka SASL authentication mechanism (`rdkafka` `sasl.mechanism`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum KafkaSaslMechanism {
    #[serde(rename = "PLAIN", alias = "plain", alias = "Plain")]
    Plain,
    #[serde(rename = "SCRAM-SHA-256", alias = "scram-sha-256", alias = "scram_sha_256")]
    ScramSha256,
    #[serde(rename = "SCRAM-SHA-512", alias = "scram-sha-512", alias = "scram_sha_512")]
    ScramSha512,
    #[serde(rename = "OAUTHBEARER", alias = "oauthbearer", alias = "Oauthbearer")]
    Oauthbearer,
}

impl KafkaSaslMechanism {
    /// Returns the exact librdkafka `sasl.mechanism` property value.
    pub fn as_rdkafka_str(&self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::ScramSha256 => "SCRAM-SHA-256",
            Self::ScramSha512 => "SCRAM-SHA-512",
            Self::Oauthbearer => "OAUTHBEARER",
        }
    }
}

/// Config-driven Apache Kafka event sink configuration (off by default).
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct KafkaEventsConfig {
    /// Disabled by default; when `false`, no Kafka client or network socket is created.
    #[serde(default)]
    pub enabled: bool,
    /// Comma-separated bootstrap servers (`host:port,...`). Required when `enabled` is true.
    #[serde(default)]
    pub brokers: String,
    /// Target topic when `topic_strategy` is `Single`.
    #[serde(default = "KafkaEventsConfig::default_topic")]
    pub topic: String,
    /// Topic prefix when `topic_strategy` is `ByDomain` or `ByCategory`.
    #[serde(default = "KafkaEventsConfig::default_topic_prefix")]
    pub topic_prefix: String,
    /// Topic naming strategy.
    #[serde(default)]
    pub topic_strategy: KafkaTopicStrategy,
    /// Partition key selection strategy.
    #[serde(default)]
    pub key_strategy: KafkaKeyStrategy,
    /// Failure handling strategy when Kafka is unreachable or busy.
    #[serde(default)]
    pub failure_mode: KafkaFailureMode,
    /// Producer acknowledgment setting (`all`, `-1`, `0`, `1`).
    #[serde(default = "KafkaEventsConfig::default_acks")]
    pub acks: String,
    /// Producer compression codec (`none`, `gzip`, `snappy`, `lz4`, `zstd`).
    #[serde(default = "KafkaEventsConfig::default_compression")]
    pub compression: String,
    /// Micro-batch linger duration in milliseconds.
    #[serde(default = "KafkaEventsConfig::default_linger_ms")]
    pub linger_ms: u32,
    /// Bounded async enqueue capacity in memory before dropping or spooling.
    #[serde(default = "KafkaEventsConfig::default_queue_capacity")]
    pub queue_capacity: usize,
    /// Broker request timeout in milliseconds.
    #[serde(default = "KafkaEventsConfig::default_request_timeout_ms")]
    pub request_timeout_ms: u32,
    /// Wire security protocol.
    #[serde(default)]
    pub security_protocol: KafkaSecurityProtocol,
    /// Optional SASL mechanism.
    #[serde(default)]
    pub sasl_mechanism: Option<KafkaSaslMechanism>,
    /// Optional SASL username.
    #[serde(default)]
    pub sasl_username: Option<String>,
    /// Optional SASL password (falls back to `YMIR_EVENTS_KAFKA_SASL_PASSWORD`).
    #[serde(default)]
    pub sasl_password: Option<String>,
    /// Optional CA certificate file path.
    #[serde(default)]
    pub ssl_ca_location: Option<String>,
    /// Optional mTLS client certificate path.
    #[serde(default)]
    pub ssl_certificate_location: Option<String>,
    /// Optional mTLS client private key path.
    #[serde(default)]
    pub ssl_key_location: Option<String>,
    /// Optional mTLS private key passphrase (falls back to `YMIR_EVENTS_KAFKA_SSL_KEY_PASSWORD`).
    #[serde(default)]
    pub ssl_key_password: Option<String>,
}

impl KafkaEventsConfig {
    fn default_topic() -> String {
        "dataspace.events".to_string()
    }

    fn default_topic_prefix() -> String {
        "dataspace.events".to_string()
    }

    fn default_acks() -> String {
        "all".to_string()
    }

    fn default_compression() -> String {
        "none".to_string()
    }

    fn default_linger_ms() -> u32 {
        10
    }

    fn default_queue_capacity() -> usize {
        4096
    }

    fn default_request_timeout_ms() -> u32 {
        5000
    }

    /// Resolves the SASL password from config or environment without storing env defaults.
    pub fn resolve_sasl_password(&self) -> Option<String> {
        EventsConfig::non_empty(self.sasl_password.as_deref())
            .or_else(|| EventsConfig::non_empty_env(&["YMIR_EVENTS_KAFKA_SASL_PASSWORD"]))
    }

    /// Resolves the SSL key password from config or environment.
    pub fn resolve_ssl_key_password(&self) -> Option<String> {
        EventsConfig::non_empty(self.ssl_key_password.as_deref())
            .or_else(|| EventsConfig::non_empty_env(&["YMIR_EVENTS_KAFKA_SSL_KEY_PASSWORD"]))
    }

    /// Validates Kafka settings whenever `enabled` is true.
    pub fn validate(&self) -> Outcome<()> {
        if !self.enabled {
            return Ok(());
        }
        if self.brokers.trim().is_empty() {
            return Err(Errors::validation(
                "events.kafka.brokers cannot be empty when kafka is enabled",
                None,
            ));
        }
        if self.topic.trim().is_empty() || self.topic_prefix.trim().is_empty() {
            return Err(Errors::validation(
                "events.kafka topic settings cannot be empty",
                None,
            ));
        }
        if self.queue_capacity == 0 {
            return Err(Errors::validation(
                "events.kafka.queue_capacity must be positive",
                None,
            ));
        }
        if !matches!(self.acks.as_str(), "all" | "-1" | "0" | "1") {
            return Err(Errors::validation(
                "events.kafka.acks must be one of all, -1, 0, 1",
                None,
            ));
        }
        if !matches!(self.compression.as_str(), "none" | "gzip" | "snappy" | "lz4" | "zstd") {
            return Err(Errors::validation(
                "events.kafka.compression has an unsupported codec",
                None,
            ));
        }
        Ok(())
    }
}

impl Default for KafkaEventsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            brokers: String::new(),
            topic: Self::default_topic(),
            topic_prefix: Self::default_topic_prefix(),
            topic_strategy: KafkaTopicStrategy::Single,
            key_strategy: KafkaKeyStrategy::CorrelationOrSubject,
            failure_mode: KafkaFailureMode::BestEffortWithMetrics,
            acks: Self::default_acks(),
            compression: Self::default_compression(),
            linger_ms: Self::default_linger_ms(),
            queue_capacity: Self::default_queue_capacity(),
            request_timeout_ms: Self::default_request_timeout_ms(),
            security_protocol: KafkaSecurityProtocol::Plaintext,
            sasl_mechanism: None,
            sasl_username: None,
            sasl_password: None,
            ssl_ca_location: None,
            ssl_certificate_location: None,
            ssl_key_location: None,
            ssl_key_password: None,
        }
    }
}

impl fmt::Debug for KafkaEventsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KafkaEventsConfig")
            .field("enabled", &self.enabled)
            .field("brokers", &self.brokers)
            .field("topic", &self.topic)
            .field("topic_prefix", &self.topic_prefix)
            .field("topic_strategy", &self.topic_strategy)
            .field("key_strategy", &self.key_strategy)
            .field("failure_mode", &self.failure_mode)
            .field("acks", &self.acks)
            .field("compression", &self.compression)
            .field("linger_ms", &self.linger_ms)
            .field("queue_capacity", &self.queue_capacity)
            .field("request_timeout_ms", &self.request_timeout_ms)
            .field("security_protocol", &self.security_protocol)
            .field("sasl_mechanism", &self.sasl_mechanism)
            .field("sasl_username", &self.sasl_username)
            .field("sasl_password", &self.sasl_password.as_ref().map(|_| "[REDACTED]"))
            .field("ssl_ca_location", &self.ssl_ca_location)
            .field("ssl_certificate_location", &self.ssl_certificate_location)
            .field("ssl_key_location", &self.ssl_key_location)
            .field("ssl_key_password", &self.ssl_key_password.as_ref().map(|_| "[REDACTED]"))
            .finish()
    }
}

/// Configuration for emitting canonical events through structured `tracing` logs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StdoutEventsConfig {
    /// When true, writes canonical events as structured tracing records for log shippers.
    #[serde(default)]
    pub enabled: bool,
    /// Minimum event severity written to stdout.
    #[serde(default = "StdoutEventsConfig::default_min_severity")]
    pub min_severity: EventSeverityLevel,
}

impl StdoutEventsConfig {
    fn default_min_severity() -> EventSeverityLevel {
        EventSeverityLevel::Info
    }
}

impl Default for StdoutEventsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_severity: Self::default_min_severity(),
        }
    }
}

/// Configuration for forwarding events from standalone agents to a central event bus API.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RemoteBusEventsConfig {
    /// Disabled by default; enabled on microservices that push into another node's bus.
    #[serde(default)]
    pub enabled: bool,
    /// Base URL of the central event bus endpoint (e.g. `http://monolith:8080/api/v1/events`).
    #[serde(default)]
    pub endpoint: Option<String>,
    /// Optional bearer token (falls back to `YMIR_EVENTS_REMOTE_BUS_TOKEN`).
    #[serde(default)]
    pub bearer_token: Option<String>,
    /// HTTP request timeout in milliseconds.
    #[serde(default = "RemoteBusEventsConfig::default_timeout_ms")]
    pub timeout_ms: u64,
}

impl RemoteBusEventsConfig {
    fn default_timeout_ms() -> u64 {
        3000
    }

    /// Resolves the bearer token from config or environment.
    pub fn resolve_bearer_token(&self) -> Option<String> {
        EventsConfig::non_empty(self.bearer_token.as_deref())
            .or_else(|| EventsConfig::non_empty_env(&["YMIR_EVENTS_REMOTE_BUS_TOKEN"]))
    }

    /// Validates endpoint syntax whenever `enabled` is true.
    pub fn validate(&self) -> Outcome<()> {
        if !self.enabled {
            return Ok(());
        }
        let raw = EventsConfig::non_empty(self.endpoint.as_deref()).ok_or_else(|| {
            Errors::validation(
                "events.remote_bus.endpoint is required when remote_bus is enabled",
                None,
            )
        })?;
        let parsed = Url::parse(&raw)
            .map_err(|_| Errors::validation("events.remote_bus.endpoint must be a valid URL", None))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(Errors::validation(
                "events.remote_bus.endpoint must use http or https",
                None,
            ));
        }
        Ok(())
    }
}

impl Default for RemoteBusEventsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            bearer_token: None,
            timeout_ms: Self::default_timeout_ms(),
        }
    }
}

impl fmt::Debug for RemoteBusEventsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteBusEventsConfig")
            .field("enabled", &self.enabled)
            .field("endpoint", &self.endpoint)
            .field("bearer_token", &self.bearer_token.as_ref().map(|_| "[REDACTED]"))
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

/// Immutable snapshot of the emitting node's runtime identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeIdentitySnapshot {
    pub dataspace_id: Option<String>,
    pub service: String,
    pub instance_id: String,
    pub participant_id: Option<String>,
}

/// Thread-safe runtime identity updated once the agent resolves its own participant DID.
#[derive(Debug)]
pub struct EventNodeIdentity {
    dataspace_id: Option<String>,
    has_explicit_service: bool,
    service: RwLock<String>,
    instance_id: String,
    participant_id: RwLock<Option<String>>,
}

impl EventNodeIdentity {
    /// Initializes identity from `EventsConfig` with a fallback service name.
    pub fn from_config(config: &EventsConfig, default_service: &str) -> Self {
        let generated_instance = Uuid::new_v4().to_string();
        let has_explicit_service = EventsConfig::non_empty(config.service_name.as_deref()).is_some()
            || EventsConfig::non_empty_env(&["YMIR_SERVICE_NAME", "OTEL_SERVICE_NAME"]).is_some();
        Self {
            dataspace_id: config.resolve_dataspace_id(),
            has_explicit_service,
            service: RwLock::new(config.resolve_service_name(default_service)),
            instance_id: config.resolve_instance_id(&generated_instance),
            participant_id: RwLock::new(None),
        }
    }

    /// Sets the boot process default service name when config/env did not pin one.
    pub fn set_default_service(&self, service: &str) {
        if self.has_explicit_service {
            return;
        }
        self.set_service(service);
    }

    /// Updates the logical service name unconditionally.
    pub fn set_service(&self, service: impl Into<String>) {
        if let Some(clean) = EventsConfig::non_empty(Some(&service.into())) {
            if let Ok(mut guard) = self.service.write() {
                *guard = clean;
            }
        }
    }

    /// Updates the local participant DID/identifier after wallet or gatekeeper boot.
    pub fn set_participant_id(&self, participant_id: Option<String>) {
        let clean = EventsConfig::non_empty(participant_id.as_deref());
        if let Ok(mut guard) = self.participant_id.write() {
            *guard = clean;
        }
    }

    /// Captures a point-in-time snapshot of node identity fields for event envelopes.
    pub fn snapshot(&self) -> NodeIdentitySnapshot {
        let service = self
            .service
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| "ymir-agent".to_string());
        let participant_id = self.participant_id.read().ok().and_then(|guard| guard.clone());
        NodeIdentitySnapshot {
            dataspace_id: self.dataspace_id.clone(),
            service,
            instance_id: self.instance_id.clone(),
            participant_id,
        }
    }
}
