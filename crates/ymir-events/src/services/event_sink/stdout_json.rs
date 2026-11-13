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

//! Structured stdout/`tracing` sink emitting canonical JSON envelopes for log collectors.

use async_trait::async_trait;
use tracing::{debug, error, info, warn};
use ymir::config::types::StdoutEventsConfig;
use ymir::errors::Outcome;

use crate::entities::envelope::{EventEnvelope, EventSeverity};
use crate::services::event_sink::EventSinkTrait;

/// Writes canonical event fields and payload JSON to `tracing` for container log shippers.
#[derive(Debug, Clone)]
pub struct TracingJsonSink {
    config: StdoutEventsConfig,
}

impl TracingJsonSink {
    /// Builds a structured log sink using the provided severity threshold.
    pub fn new(config: StdoutEventsConfig) -> Self {
        Self { config }
    }

    fn meets_threshold(&self, severity: EventSeverity) -> bool {
        severity >= EventSeverity::from(self.config.min_severity)
    }

    fn record_event(&self, envelope: &EventEnvelope) {
        let json = serde_json::to_string(envelope).unwrap_or_else(|_| "{}".to_string());
        match envelope.severity {
            EventSeverity::Debug => debug!(
                target: "ymir.events.audit",
                event_id = %envelope.id,
                topic = %envelope.topic,
                event_type = %envelope.event_type,
                payload_type = %envelope.payload_type,
                service = %envelope.source.service,
                participant_id = ?envelope.source.participant_id,
                correlation_id = ?envelope.correlation.correlation_id,
                envelope = %json,
                "dataspace event"
            ),
            EventSeverity::Info => info!(
                target: "ymir.events.audit",
                event_id = %envelope.id,
                topic = %envelope.topic,
                event_type = %envelope.event_type,
                payload_type = %envelope.payload_type,
                service = %envelope.source.service,
                participant_id = ?envelope.source.participant_id,
                correlation_id = ?envelope.correlation.correlation_id,
                envelope = %json,
                "dataspace event"
            ),
            EventSeverity::Warn => warn!(
                target: "ymir.events.audit",
                event_id = %envelope.id,
                topic = %envelope.topic,
                event_type = %envelope.event_type,
                payload_type = %envelope.payload_type,
                service = %envelope.source.service,
                participant_id = ?envelope.source.participant_id,
                correlation_id = ?envelope.correlation.correlation_id,
                envelope = %json,
                "dataspace event"
            ),
            EventSeverity::Error | EventSeverity::Critical => error!(
                target: "ymir.events.audit",
                event_id = %envelope.id,
                topic = %envelope.topic,
                event_type = %envelope.event_type,
                payload_type = %envelope.payload_type,
                service = %envelope.source.service,
                participant_id = ?envelope.source.participant_id,
                correlation_id = ?envelope.correlation.correlation_id,
                envelope = %json,
                "dataspace event"
            ),
        }
    }
}

#[async_trait]
impl EventSinkTrait for TracingJsonSink {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        if self.meets_threshold(envelope.severity) {
            self.record_event(&envelope);
        }
        Ok(())
    }
}
