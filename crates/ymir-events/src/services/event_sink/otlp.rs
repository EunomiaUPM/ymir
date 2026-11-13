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

//! OpenTelemetry Logs sink exporting canonical event envelopes as structured `LogRecord`s.

use std::time::SystemTime;

use async_trait::async_trait;
use opentelemetry::logs::{AnyValue, LogRecord, Logger, LoggerProvider};
use opentelemetry::trace::{SpanContext, TraceContextExt};
use opentelemetry_sdk::logs::{SdkLogRecord, SdkLoggerProvider};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use ymir::config::types::OtlpEventsConfig;
use ymir::errors::Outcome;
use ymir::telemetry::{Telemetry, TraceParent};

use crate::entities::envelope::{EventEnvelope, EventSeverity};
use crate::services::event_sink::EventSinkTrait;

/// Maps canonical [`EventEnvelope`] events onto OpenTelemetry Logs (`SdkLoggerProvider`).
#[derive(Clone, Debug)]
pub struct OtlpEventSink {
    provider: SdkLoggerProvider,
    config: OtlpEventsConfig,
    scope_name: &'static str,
}

impl OtlpEventSink {
    /// Builds an OTLP sink bound to an explicit `SdkLoggerProvider`.
    pub fn new(provider: SdkLoggerProvider, config: OtlpEventsConfig) -> Self {
        Self {
            provider,
            config,
            scope_name: "ymir-events",
        }
    }

    /// Builds an OTLP sink from the global `Telemetry` logger provider when OTLP logs are active.
    pub fn from_global(config: &OtlpEventsConfig) -> Option<Self> {
        Telemetry::logger_provider().map(|provider| Self::new(provider, config.clone()))
    }

    fn resolve_trace_context(envelope: &EventEnvelope) -> Option<SpanContext> {
        if let Some(traceparent) = &envelope.correlation.trace_context {
            if let Some(ctx) = TraceParent::parse_context(traceparent) {
                return Some(ctx);
            }
        }
        let current = Span::current().context().span().span_context().clone();
        current.is_valid().then_some(current)
    }

    fn populate_record(&self, record: &mut SdkLogRecord, envelope: &EventEnvelope) {
        record.set_timestamp(SystemTime::from(envelope.timestamp));
        record.set_observed_timestamp(SystemTime::now());
        record.set_severity_number(envelope.severity.to_otel_severity());
        record.set_severity_text(envelope.severity.as_otel_text());
        record.set_event_name("dataspace.event");

        let body = if self.config.include_full_payload {
            envelope.payload.to_string()
        } else {
            format!("{} ({})", envelope.topic, envelope.payload_type)
        };
        record.set_body(AnyValue::from(body));

        if let Some(span_ctx) = Self::resolve_trace_context(envelope) {
            record.set_trace_context(
                span_ctx.trace_id(),
                span_ctx.span_id(),
                Some(span_ctx.trace_flags()),
            );
        }

        Self::add_core_attributes(record, envelope);
        Self::add_actor_and_subject_attributes(record, envelope);
        Self::add_correlation_attributes(record, envelope);

        if self.config.include_full_payload {
            record.add_attribute("event.payload", envelope.payload.to_string());
        }
    }

    fn add_core_attributes(record: &mut SdkLogRecord, envelope: &EventEnvelope) {
        record.add_attribute("event.id", envelope.id.to_string());
        record.add_attribute("event.spec_version", envelope.spec_version.clone());
        record.add_attribute("event.topic", envelope.topic.as_str().to_string());
        record.add_attribute("event.category", envelope.event_type.as_str());
        record.add_attribute("event.severity", envelope.severity.as_str());
        record.add_attribute("event.delivery_class", envelope.delivery_class.as_str());
        record.add_attribute("event.payload_type", envelope.payload_type.clone());
        record.add_attribute("event.schema_version", envelope.schema_version as i64);
        record.add_attribute(
            "event.integrity.payload_digest",
            envelope.integrity.payload_digest.clone(),
        );
        if let Some(att) = &envelope.integrity.attestation {
            record.add_attribute("event.integrity.attestation.format", att.format.clone());
            record.add_attribute(
                "event.integrity.attestation.signer_did",
                att.signer_did.clone(),
            );
            record.add_attribute(
                "event.integrity.attestation.signature",
                att.signature.clone(),
            );
        }
        record.add_attribute("event.source.service", envelope.source.service.clone());
        record.add_attribute(
            "event.source.instance_id",
            envelope.source.instance_id.clone(),
        );
        record.add_attribute("event.source.crate", envelope.source.crate_name.clone());

        if let Some(outcome) = envelope.outcome {
            record.add_attribute("event.outcome", outcome.as_str());
        }
        if let Some(dataspace_id) = &envelope.source.dataspace_id {
            record.add_attribute("dataspace.id", dataspace_id.clone());
        }
        if let Some(participant_id) = &envelope.source.participant_id {
            record.add_attribute("participant.id", participant_id.clone());
        }
    }

    fn add_actor_and_subject_attributes(record: &mut SdkLogRecord, envelope: &EventEnvelope) {
        record.add_attribute("actor.type", envelope.actor.actor_type.as_str());
        if !envelope.actor.id.is_empty() {
            record.add_attribute("actor.id", envelope.actor.id.clone());
        }
        if let Some(actor_pid) = &envelope.actor.participant_id {
            record.add_attribute("actor.participant_id", actor_pid.clone());
        }
        if let Some(role) = &envelope.actor.role {
            record.add_attribute("actor.role", role.clone());
        }
        if let Some(subject) = &envelope.subject {
            record.add_attribute("subject.kind", subject.kind.clone());
            record.add_attribute("subject.id", subject.id.clone());
            if let Some(parent_id) = &subject.parent_id {
                record.add_attribute("subject.parent_id", parent_id.clone());
            }
        }
    }

    fn add_correlation_attributes(record: &mut SdkLogRecord, envelope: &EventEnvelope) {
        if let Some(corr) = &envelope.correlation.correlation_id {
            record.add_attribute("correlation.id", corr.clone());
        }
        if let Some(causation) = &envelope.correlation.causation_id {
            record.add_attribute("causation.id", causation.clone());
        }
        let refs = &envelope.correlation.flow_refs;
        Self::add_optional(record, "flow.agreement_id", refs.agreement_id.as_ref());
        Self::add_optional(record, "flow.transfer_id", refs.transfer_id.as_ref());
        Self::add_optional(record, "flow.negotiation_id", refs.negotiation_id.as_ref());
        Self::add_optional(record, "flow.grant_id", refs.grant_id.as_ref());
        Self::add_optional(record, "flow.dataset_id", refs.dataset_id.as_ref());
        Self::add_optional(record, "flow.peer_id", refs.peer_id.as_ref());
    }

    fn add_optional(record: &mut SdkLogRecord, key: &'static str, value: Option<&String>) {
        if let Some(val) = value {
            record.add_attribute(key, val.clone());
        }
    }
}

#[async_trait]
impl EventSinkTrait for OtlpEventSink {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        if !self.config.enabled || envelope.severity < EventSeverity::from(self.config.min_severity) {
            return Ok(());
        }
        let logger = self.provider.logger(self.scope_name);
        let mut record = logger.create_log_record();
        self.populate_record(&mut record, &envelope);
        logger.emit(record);
        Ok(())
    }
}
