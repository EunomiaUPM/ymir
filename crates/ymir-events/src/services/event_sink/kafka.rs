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

//! Config-driven Apache Kafka routing, async producer sink, and durable outbox relay worker.

use std::sync::{Arc, LazyLock};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use opentelemetry::metrics::Counter;
use opentelemetry::{KeyValue, global};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};
use uuid::Uuid;
use ymir::config::types::{KafkaEventsConfig, KafkaFailureMode, KafkaKeyStrategy, KafkaTopicStrategy};
use ymir::errors::Outcome;

use crate::data::repo::EventOutboxRepo;
use crate::entities::envelope::{DeliveryClass, EventEnvelope};
use crate::entities::outbox::{OutboxRecord, OutboxStatus};
use crate::services::event_sink::EventSinkTrait;

static KAFKA_SENT: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter("events")
        .u64_counter("events.kafka.sent")
        .build()
});

static KAFKA_FAILED: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter("events")
        .u64_counter("events.kafka.failed")
        .build()
});

static KAFKA_SPOOLED: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter("events")
        .u64_counter("events.kafka.outbox_spooled")
        .build()
});

/// Serialized Kafka record handed to the producer port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KafkaMessage {
    pub topic: String,
    pub key: String,
    pub payload: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

/// Port wrapping Kafka production for unit testing without a live broker.
#[mockall::automock]
#[async_trait]
pub trait KafkaProducerPort: Send + Sync + 'static {
    async fn produce(&self, msg: KafkaMessage) -> Result<(), String>;
}

/// Pure topic, key and header derivation independent of `rdkafka`.
pub struct KafkaRouting;

impl KafkaRouting {
    /// Resolves destination Kafka topic from `config.topic_strategy` and `envelope`.
    pub fn resolve_topic(config: &KafkaEventsConfig, envelope: &EventEnvelope) -> String {
        match config.topic_strategy {
            KafkaTopicStrategy::Single => config.topic.clone(),
            KafkaTopicStrategy::ByDomain => {
                let first = envelope
                    .topic
                    .segments()
                    .into_iter()
                    .next()
                    .unwrap_or("general");
                let slug = Self::sanitize_segment(first);
                format!("{}.{}", config.topic_prefix, slug)
            }
            KafkaTopicStrategy::ByCategory => {
                format!("{}.{}", config.topic_prefix, envelope.event_type.as_kebab_str())
            }
        }
    }

    /// Resolves Kafka partition key from `config.key_strategy` and `envelope`.
    pub fn resolve_key(config: &KafkaEventsConfig, envelope: &EventEnvelope) -> String {
        match config.key_strategy {
            KafkaKeyStrategy::CorrelationOrSubject => envelope
                .correlation
                .correlation_id
                .clone()
                .or_else(|| {
                    envelope
                        .correlation
                        .flow_refs
                        .primary_correlation()
                        .map(ToOwned::to_owned)
                })
                .or_else(|| envelope.subject.as_ref().map(|s| s.id.clone()))
                .unwrap_or_else(|| envelope.id.to_string()),
            KafkaKeyStrategy::ParticipantId => envelope
                .source
                .participant_id
                .clone()
                .unwrap_or_else(|| envelope.id.to_string()),
            KafkaKeyStrategy::EventId => envelope.id.to_string(),
        }
    }

    /// Builds normalized metadata headers attached to each Kafka message.
    pub fn build_headers(envelope: &EventEnvelope) -> Vec<(String, String)> {
        let mut headers = vec![
            ("spec_version".to_string(), envelope.spec_version.clone()),
            ("topic".to_string(), envelope.topic.as_str().to_string()),
            (
                "event_type".to_string(),
                envelope.event_type.as_str().to_string(),
            ),
            (
                "severity".to_string(),
                envelope.severity.as_str().to_string(),
            ),
            (
                "payload_type".to_string(),
                envelope.payload_type.clone(),
            ),
            (
                "delivery_class".to_string(),
                envelope.delivery_class.as_str().to_string(),
            ),
            (
                "payload_digest".to_string(),
                envelope.integrity.payload_digest.clone(),
            ),
            ("service".to_string(), envelope.source.service.clone()),
        ];

        Self::push_opt(&mut headers, "outcome", envelope.outcome.map(|o| o.to_string()));
        Self::push_opt(&mut headers, "dataspace_id", envelope.source.dataspace_id.clone());
        Self::push_opt(
            &mut headers,
            "participant_id",
            envelope.source.participant_id.clone(),
        );
        Self::push_opt(
            &mut headers,
            "correlation_id",
            envelope.correlation.correlation_id.clone(),
        );
        Self::push_opt(
            &mut headers,
            "traceparent",
            envelope.correlation.trace_context.clone(),
        );
        headers
    }

    /// Constructs a [`KafkaMessage`] containing serialized JSON envelope and routing metadata.
    pub fn build_message(
        config: &KafkaEventsConfig,
        envelope: &EventEnvelope,
    ) -> Result<KafkaMessage, String> {
        let payload = serde_json::to_vec(envelope).map_err(|e| e.to_string())?;
        Ok(KafkaMessage {
            topic: Self::resolve_topic(config, envelope),
            key: Self::resolve_key(config, envelope),
            payload,
            headers: Self::build_headers(envelope),
        })
    }

    /// Computes exponential retry timestamp bounded at one hour for outbox backoff.
    pub fn next_backoff(now: DateTime<Utc>, attempts: u32) -> DateTime<Utc> {
        let exp = attempts.saturating_sub(1).min(10);
        let secs = (5_i64.saturating_mul(1_i64 << exp)).min(3600);
        now + chrono::Duration::seconds(secs)
    }

    fn sanitize_segment(raw: &str) -> String {
        let cleaned: String = raw
            .trim()
            .to_ascii_lowercase()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else {
                    '-'
                }
            })
            .collect();
        if cleaned.is_empty() {
            "general".to_string()
        } else {
            cleaned
        }
    }

    fn push_opt(headers: &mut Vec<(String, String)>, key: &'static str, val: Option<String>) {
        if let Some(v) = val.filter(|s| !s.is_empty()) {
            headers.push((key.to_string(), v));
        }
    }
}

/// Non-blocking Kafka event sink supporting `BestEffortWithMetrics` and `OutboxRelay`.
#[derive(Clone)]
pub struct KafkaEventSink {
    config: KafkaEventsConfig,
    producer: Arc<dyn KafkaProducerPort>,
    outbox_repo: Option<Arc<dyn EventOutboxRepo>>,
    permits: Arc<Semaphore>,
}

impl KafkaEventSink {
    /// Builds a sink wrapping any `KafkaProducerPort` (usable in tests without `rdkafka`).
    pub fn with_producer(
        config: KafkaEventsConfig,
        producer: Arc<dyn KafkaProducerPort>,
        outbox_repo: Option<Arc<dyn EventOutboxRepo>>,
    ) -> Self {
        let capacity = config.queue_capacity.max(16);
        Self {
            config,
            producer,
            outbox_repo,
            permits: Arc::new(Semaphore::new(capacity)),
        }
    }

    /// Produces `envelope` synchronously inside the current future, spooling failures if configured.
    pub async fn deliver_envelope(&self, envelope: EventEnvelope) -> Result<(), String> {
        let message = KafkaRouting::build_message(&self.config, &envelope)?;
        Self::dispatch_message(
            self.producer.as_ref(),
            self.config.failure_mode,
            self.outbox_repo.as_deref(),
            &envelope,
            message,
        )
        .await
    }

    async fn dispatch_message(
        producer: &dyn KafkaProducerPort,
        failure_mode: KafkaFailureMode,
        outbox_repo: Option<&dyn EventOutboxRepo>,
        envelope: &EventEnvelope,
        message: KafkaMessage,
    ) -> Result<(), String> {
        let topic = message.topic.clone();
        let key = message.key.clone();
        match producer.produce(message).await {
            Ok(()) => {
                KAFKA_SENT.add(1, &[KeyValue::new("kafka.topic", topic)]);
                Ok(())
            }
            Err(err) => {
                KAFKA_FAILED.add(1, &[KeyValue::new("kafka.topic", topic.clone())]);
                warn!(
                    topic = %topic,
                    event_id = %envelope.id,
                    error = %err,
                    "Kafka event delivery failed"
                );
                Self::maybe_spool_outbox(failure_mode, outbox_repo, envelope, &topic, &key, &err)
                    .await;
                Err(err)
            }
        }
    }

    async fn maybe_spool_outbox(
        failure_mode: KafkaFailureMode,
        outbox_repo: Option<&dyn EventOutboxRepo>,
        envelope: &EventEnvelope,
        topic: &str,
        key: &str,
        reason: &str,
    ) {
        if failure_mode != KafkaFailureMode::OutboxRelay
            || envelope.delivery_class != DeliveryClass::AuditDurable
        {
            return;
        }
        let Some(repo) = outbox_repo else {
            return;
        };
        let Ok(envelope_json) = serde_json::to_value(envelope) else {
            return;
        };
        let now = Utc::now();
        let record = OutboxRecord {
            id: format!("urn:uuid:{}", Uuid::new_v4()),
            sink_name: "kafka".to_string(),
            event_id: envelope.id.to_string(),
            destination: topic.to_string(),
            partition_key: key.to_string(),
            envelope_json,
            attempts: 1,
            next_retry_at: KafkaRouting::next_backoff(now, 1),
            last_error: Some(reason.to_string()),
            status: OutboxStatus::Pending,
            created_at: now,
            delivered_at: None,
        };
        if let Err(err) = repo.enqueue(&record).await {
            warn!(event_id = %envelope.id, error = ?err, "Failed to spool Kafka event to outbox");
        } else {
            KAFKA_SPOOLED.add(1, &[KeyValue::new("kafka.topic", topic.to_string())]);
        }
    }
}

#[async_trait]
impl EventSinkTrait for KafkaEventSink {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        if !self.config.enabled {
            return Ok(());
        }
        let message = match KafkaRouting::build_message(&self.config, &envelope) {
            Ok(msg) => msg,
            Err(err) => {
                warn!(event_id = %envelope.id, error = %err, "Failed to serialize Kafka event");
                return Ok(());
            }
        };

        let Ok(permit) = self.permits.clone().try_acquire_owned() else {
            KAFKA_FAILED.add(1, &[KeyValue::new("kafka.topic", message.topic.clone())]);
            warn!(
                topic = %message.topic,
                event_id = %envelope.id,
                "Kafka async queue saturated; applying failure mode"
            );
            Self::maybe_spool_outbox(
                self.config.failure_mode,
                self.outbox_repo.as_deref(),
                &envelope,
                &message.topic,
                &message.key,
                "kafka async queue saturated",
            )
            .await;
            return Ok(());
        };

        let producer = self.producer.clone();
        let failure_mode = self.config.failure_mode;
        let outbox_repo = self.outbox_repo.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let _ = Self::dispatch_message(
                producer.as_ref(),
                failure_mode,
                outbox_repo.as_deref(),
                &envelope,
                message,
            )
            .await;
        });
        Ok(())
    }
}

/// Replays pending `event_outbox` rows through any [`KafkaProducerPort`].
#[derive(Clone)]
pub struct KafkaOutboxWorker {
    producer: Arc<dyn KafkaProducerPort>,
    outbox_repo: Arc<dyn EventOutboxRepo>,
    poll_interval: Duration,
    max_attempts: u32,
}

impl KafkaOutboxWorker {
    /// Creates an outbox relay worker for durable Kafka retries.
    pub fn with_producer(
        producer: Arc<dyn KafkaProducerPort>,
        outbox_repo: Arc<dyn EventOutboxRepo>,
        poll_interval: Duration,
        max_attempts: u32,
    ) -> Self {
        Self {
            producer,
            outbox_repo,
            poll_interval,
            max_attempts: max_attempts.max(1),
        }
    }

    /// Replays one batch of due outbox records; returns number successfully delivered.
    pub async fn poll_once(&self) -> Outcome<usize> {
        let due = self.outbox_repo.get_due_pending(Utc::now(), 50).await?;
        let mut delivered = 0usize;
        for record in due {
            if self.relay_record(record).await {
                delivered += 1;
            }
        }
        Ok(delivered)
    }

    async fn relay_record(&self, record: OutboxRecord) -> bool {
        let payload = match serde_json::to_vec(&record.envelope_json) {
            Ok(bytes) => bytes,
            Err(err) => {
                let _ = self
                    .outbox_repo
                    .mark_dead_letter(&record.id, record.attempts + 1, &err.to_string())
                    .await;
                return false;
            }
        };

        let headers = serde_json::from_value::<EventEnvelope>(record.envelope_json.clone())
            .map(|env| KafkaRouting::build_headers(&env))
            .unwrap_or_default();

        let msg = KafkaMessage {
            topic: record.destination.clone(),
            key: record.partition_key.clone(),
            payload,
            headers,
        };

        let next_attempt = record.attempts + 1;
        match self.producer.produce(msg).await {
            Ok(()) => {
                KAFKA_SENT.add(1, &[KeyValue::new("kafka.topic", record.destination)]);
                let _ = self.outbox_repo.mark_delivered(&record.id, next_attempt).await;
                true
            }
            Err(err) => {
                KAFKA_FAILED.add(1, &[KeyValue::new("kafka.topic", record.destination.clone())]);
                if next_attempt >= self.max_attempts {
                    let _ = self
                        .outbox_repo
                        .mark_dead_letter(&record.id, next_attempt, &err)
                        .await;
                } else {
                    let retry_at = KafkaRouting::next_backoff(Utc::now(), next_attempt);
                    let _ = self
                        .outbox_repo
                        .reschedule(&record.id, next_attempt, retry_at, &err)
                        .await;
                }
                false
            }
        }
    }
}

#[async_trait]
impl ymir::boot::workers::BackgroundWorker for KafkaOutboxWorker {
    fn name(&self) -> &'static str {
        "kafka-event-outbox"
    }

    async fn run(
        self: Box<Self>,
        token: tokio_util::sync::CancellationToken,
    ) -> Outcome<()> {
        info!(interval_secs = self.poll_interval.as_secs(), "Kafka outbox worker started");
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    info!("Kafka outbox worker shutting down");
                    return Ok(());
                }
                _ = tokio::time::sleep(self.poll_interval) => {
                    match self.poll_once().await {
                        Ok(count) if count > 0 => {
                            debug!(replayed = count, "Kafka outbox batch replayed");
                        }
                        Ok(_) => {}
                        Err(err) => {
                            warn!(error = ?err, "Kafka outbox poll failed");
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "kafka")]
mod rdkafka_impl {
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use rdkafka::config::ClientConfig;
    use rdkafka::message::{Header, OwnedHeaders};
    use rdkafka::producer::{FutureProducer, FutureRecord};
    use rdkafka::util::Timeout;
    use ymir::config::types::KafkaEventsConfig;
    use ymir::errors::{Errors, Outcome};

    use crate::data::repo::EventOutboxRepo;
    use crate::services::event_sink::kafka::{
        KafkaEventSink, KafkaMessage, KafkaOutboxWorker, KafkaProducerPort,
    };

    /// `rdkafka::FutureProducer` implementation of [`KafkaProducerPort`].
    #[derive(Clone)]
    pub struct RdkafkaProducerAdapter {
        producer: FutureProducer,
        request_timeout: Duration,
    }

    impl RdkafkaProducerAdapter {
        /// Builds an `rdkafka` `FutureProducer` configured from `config`.
        pub fn from_config(config: &KafkaEventsConfig) -> Outcome<Self> {
            config.validate()?;
            let mut client = ClientConfig::new();
            client
                .set("bootstrap.servers", &config.brokers)
                .set("acks", &config.acks)
                .set("compression.type", &config.compression)
                .set("linger.ms", config.linger_ms.to_string())
                .set("request.timeout.ms", config.request_timeout_ms.to_string())
                .set("message.timeout.ms", config.request_timeout_ms.to_string())
                .set("security.protocol", config.security_protocol.as_rdkafka_str());

            if let Some(mech) = config.sasl_mechanism {
                client.set("sasl.mechanism", mech.as_rdkafka_str());
            }
            if let Some(user) = &config.sasl_username {
                client.set("sasl.username", user);
            }
            if let Some(pass) = config.resolve_sasl_password() {
                client.set("sasl.password", &pass);
            }
            if let Some(ca) = &config.ssl_ca_location {
                client.set("ssl.ca.location", ca);
            }
            if let Some(cert) = &config.ssl_certificate_location {
                client.set("ssl.certificate.location", cert);
            }
            if let Some(key) = &config.ssl_key_location {
                client.set("ssl.key.location", key);
            }
            if let Some(key_pw) = config.resolve_ssl_key_password() {
                client.set("ssl.key.password", &key_pw);
            }

            let producer: FutureProducer = client
                .create()
                .map_err(|e| Errors::validation(format!("failed to create Kafka producer: {e}"), None))?;

            Ok(Self {
                producer,
                request_timeout: Duration::from_millis(config.request_timeout_ms as u64),
            })
        }
    }

    #[async_trait]
    impl KafkaProducerPort for RdkafkaProducerAdapter {
        async fn produce(&self, msg: KafkaMessage) -> Result<(), String> {
            let mut owned_headers = OwnedHeaders::new();
            for (k, v) in &msg.headers {
                owned_headers = owned_headers.insert(Header {
                    key: k.as_str(),
                    value: Some(v.as_bytes()),
                });
            }

            let record = FutureRecord::to(&msg.topic)
                .key(&msg.key)
                .payload(&msg.payload)
                .headers(owned_headers);

            self.producer
                .send(record, Timeout::After(self.request_timeout))
                .await
                .map(|_| ())
                .map_err(|(err, _)| err.to_string())
        }
    }

    impl KafkaEventSink {
        /// Builds a live `rdkafka` event sink from validated configuration.
        pub fn new(
            config: KafkaEventsConfig,
            outbox_repo: Option<Arc<dyn EventOutboxRepo>>,
        ) -> Outcome<Self> {
            let adapter = Arc::new(RdkafkaProducerAdapter::from_config(&config)?);
            Ok(Self::with_producer(config, adapter, outbox_repo))
        }
    }

    impl KafkaOutboxWorker {
        /// Builds an outbox replay worker backed by `rdkafka`.
        pub fn new(
            config: &KafkaEventsConfig,
            outbox_repo: Arc<dyn EventOutboxRepo>,
        ) -> Outcome<Self> {
            let adapter = Arc::new(RdkafkaProducerAdapter::from_config(config)?);
            Ok(Self::with_producer(
                adapter,
                outbox_repo,
                Duration::from_secs(5),
                10,
            ))
        }
    }
}

#[cfg(feature = "kafka")]
pub use rdkafka_impl::RdkafkaProducerAdapter;
