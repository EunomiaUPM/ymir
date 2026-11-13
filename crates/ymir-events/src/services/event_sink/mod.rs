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

//! Event emission sinks: local store, OTLP logs, Kafka, structured stdout, and remote relay.

pub mod async_telemetry;
pub mod composite;
pub mod kafka;
pub mod noop;
pub mod otlp;
pub mod remote_bus;
pub mod stdout_json;

use async_trait::async_trait;
use ymir::errors::Outcome;

pub use async_telemetry::{AsyncTelemetrySink, TelemetryFlushWorker};
pub use composite::{BuiltEventPipeline, CompositeEventSink, EventPipelineBuilder};
pub use kafka::{KafkaMessage, KafkaProducerPort, KafkaRouting, MockKafkaProducerPort};
#[cfg(feature = "kafka")]
pub use kafka::{KafkaEventSink, KafkaOutboxWorker, RdkafkaProducerAdapter};
pub use noop::NoopEventSink;
pub use otlp::OtlpEventSink;
pub use remote_bus::RemoteBusSink;
pub use stdout_json::TracingJsonSink;

use crate::entities::envelope::{EventEnvelope, EventSource};
use crate::entities::event::{EventContext, EventPayload};

/// Destination receiving canonical v2 [`EventEnvelope`] records.
#[mockall::automock]
#[async_trait]
pub trait EventSinkTrait: Send + Sync {
    /// Emits a canonical immutable event envelope through this sink.
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()>;

    /// Emits multiple envelopes; sinks may persist or forward them as one batch.
    async fn emit_envelopes(&self, envelopes: Vec<EventEnvelope>) -> Outcome<()> {
        for envelope in envelopes {
            self.emit_envelope(envelope).await?;
        }
        Ok(())
    }
}

/// Typed convenience helpers implemented automatically for every [`EventSinkTrait`].
#[async_trait]
pub trait TypedEventSink: EventSinkTrait {
    /// Serializes `payload` into a canonical envelope stamped with `ctx` and emits it.
    async fn emit<P: EventPayload>(&self, ctx: &EventContext, payload: &P) -> Outcome<()> {
        let envelope = payload.into_envelope(ctx, EventSource::for_crate(P::CRATE_NAME))?;
        self.emit_envelope(envelope).await
    }

    /// Materializes a slice of `(ctx, payload)` pairs and emits them in one batch.
    async fn emit_batch<P: EventPayload>(
        &self,
        items: &[(&EventContext, &P)],
    ) -> Outcome<()> {
        if items.is_empty() {
            return Ok(());
        }
        let mut envelopes = Vec::with_capacity(items.len());
        let source = EventSource::for_crate(P::CRATE_NAME);
        for (ctx, payload) in items {
            envelopes.push(payload.into_envelope(ctx, source.clone())?);
        }
        self.emit_envelopes(envelopes).await
    }
}

impl<T: EventSinkTrait + ?Sized> TypedEventSink for T {}
