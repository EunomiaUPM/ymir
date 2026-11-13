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

//! Unified dataspace event envelope, typed domain schemas, and composable delivery sinks.
//!
//! Producers emit typed `EventPayload` values through `EventSinkTrait` / `TypedEventSink` (with
//! caller `EventContext`), which fan out across local Postgres/webhooks (`EventBus`),
//! OpenTelemetry Logs (`OtlpEventSink`), Apache Kafka (`KafkaEventSink`), structured stdout, or a
//! remote bus API.

pub mod data;
pub mod entities;
pub mod http;
pub mod routes;
pub mod schemas;
pub mod services;
pub mod setup;

/// Service id, also the mount path segment of the API.
pub const SERVICE_NAME: &str = "events";

pub use entities::dto::EntityDeletedDto;
pub use entities::envelope::{
    ActorType, DeliveryClass, EntityRef, EventActor, EventAttestation, EventCategory,
    EventCorrelation, EventEnvelope, EventEnvelopeDraft, EventIntegrity, EventOutcome,
    EventSeverity, EventSource, FlowRefs,
};
pub use entities::event::{EventContext, EventPayload};
pub use entities::outbox::{OutboxRecord, OutboxStatus};
pub use entities::subscription::WebhookDeliveryFormat;
pub use entities::topic::Topic;
pub use entities::topic_pattern::TopicPattern;
pub use services::event_bus::{
    DeadLetterView, DeliveryView, EventBus, EventBusTrait, EventDispatcher, EventPublisherTrait,
    EventView, RetryPolicy, RetryWorker, SubscriptionView,
};
#[cfg(feature = "kafka")]
pub use services::event_sink::{KafkaEventSink, KafkaOutboxWorker, RdkafkaProducerAdapter};
pub use services::event_sink::{
    BuiltEventPipeline, CompositeEventSink, EventPipelineBuilder, EventSinkTrait, KafkaMessage,
    KafkaProducerPort, KafkaRouting, MockEventSinkTrait, MockKafkaProducerPort, NoopEventSink,
    OtlpEventSink, RemoteBusSink, TracingJsonSink, TypedEventSink,
};
pub use setup::{EventsModule, StandaloneEventsModule};
