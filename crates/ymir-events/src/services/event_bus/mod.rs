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

//! The event bus.

pub mod dispatcher;
pub mod policy;
pub mod service;
pub mod views;
pub mod worker;

use async_trait::async_trait;
use tokio::sync::broadcast;
use ymir::errors::Outcome;

pub use dispatcher::EventDispatcher;
pub use policy::RetryPolicy;
pub use service::EventBus;
pub use views::{DeadLetterView, DeliveryView, EventView, SubscriptionView};
pub use worker::RetryWorker;

use crate::entities::envelope::EventEnvelope;
use crate::entities::event::{EventContext, EventPayload};

/// Publishing envelopes and listening to them in-process.
#[async_trait]
pub trait EventBusTrait: Send + Sync + 'static {
    /// Stores the envelope, broadcasts it and starts a delivery per matching subscription.
    async fn publish(&self, envelope: EventEnvelope) -> Outcome<EventEnvelope>;

    /// Stores and broadcasts multiple envelopes as a single batch.
    async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Outcome<Vec<EventEnvelope>> {
        let mut out = Vec::with_capacity(envelopes.len());
        for envelope in envelopes {
            out.push(self.publish(envelope).await?);
        }
        Ok(out)
    }

    /// New receiver of every envelope published from now on.
    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope>;
}

/// Publishing canonical typed payloads and raw envelopes through the local event bus.
#[async_trait]
pub trait EventPublisherTrait: Send + Sync {
    /// Materializes `payload` and `ctx` into an immutable envelope and publishes it.
    async fn publish_typed<P: EventPayload>(
        &self,
        ctx: &EventContext,
        payload: &P,
    ) -> Outcome<EventEnvelope>;

    /// Publishes an already constructed canonical [`EventEnvelope`].
    async fn emit_raw(&self, envelope: EventEnvelope) -> Outcome<EventEnvelope>;
}
