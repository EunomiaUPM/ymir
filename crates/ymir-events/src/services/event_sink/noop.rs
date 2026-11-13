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

//! No-op sink, adapters over `EventBus`, and smart pointer delegating impls.

use std::sync::Arc;

use async_trait::async_trait;
use ymir::errors::Outcome;

use crate::entities::envelope::EventEnvelope;
use crate::services::event_bus::EventBus;
use crate::services::event_sink::EventSinkTrait;

/// Drops every event: used when no sink is wired or in lightweight unit tests.
#[derive(Debug, Clone, Default)]
pub struct NoopEventSink;

impl NoopEventSink {
    /// Returns the local bus when provided, or a `NoopEventSink` otherwise.
    pub fn or_bus(bus: Option<EventBus>) -> Arc<dyn EventSinkTrait> {
        match bus {
            Some(bus) => Arc::new(bus),
            None => Arc::new(Self),
        }
    }

    /// Returns an existing sink when present, or a `NoopEventSink` otherwise.
    pub fn or_sink(sink: Option<Arc<dyn EventSinkTrait>>) -> Arc<dyn EventSinkTrait> {
        sink.unwrap_or_else(|| Arc::new(Self))
    }
}

#[async_trait]
impl EventSinkTrait for NoopEventSink {
    async fn emit_envelope(&self, _: EventEnvelope) -> Outcome<()> {
        Ok(())
    }

    async fn emit_envelopes(&self, _: Vec<EventEnvelope>) -> Outcome<()> {
        Ok(())
    }
}

#[async_trait]
impl EventSinkTrait for EventBus {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        self.publish(envelope).await?;
        Ok(())
    }

    async fn emit_envelopes(&self, envelopes: Vec<EventEnvelope>) -> Outcome<()> {
        self.publish_batch(envelopes).await?;
        Ok(())
    }
}

#[async_trait]
impl<T: EventSinkTrait> EventSinkTrait for Option<T> {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        match self {
            Some(sink) => sink.emit_envelope(envelope).await,
            None => Ok(()),
        }
    }

    async fn emit_envelopes(&self, envelopes: Vec<EventEnvelope>) -> Outcome<()> {
        match self {
            Some(sink) => sink.emit_envelopes(envelopes).await,
            None => Ok(()),
        }
    }
}

#[async_trait]
impl<T: EventSinkTrait + ?Sized> EventSinkTrait for &T {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        (**self).emit_envelope(envelope).await
    }

    async fn emit_envelopes(&self, envelopes: Vec<EventEnvelope>) -> Outcome<()> {
        (**self).emit_envelopes(envelopes).await
    }
}

#[async_trait]
impl<T: EventSinkTrait + ?Sized> EventSinkTrait for Arc<T> {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        (**self).emit_envelope(envelope).await
    }

    async fn emit_envelopes(&self, envelopes: Vec<EventEnvelope>) -> Outcome<()> {
        (**self).emit_envelopes(envelopes).await
    }
}
