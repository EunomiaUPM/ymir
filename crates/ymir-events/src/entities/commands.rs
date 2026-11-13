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

//! Create and update commands for subscriptions and published events.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::secret::Secret;
use ymir::types::participants::Visibility;

use crate::entities::envelope::{
    DeliveryClass, EntityRef, EventActor, EventCategory, EventEnvelope, EventOutcome, EventSeverity,
    EventSource, FlowRefs,
};
use crate::entities::subscription::WebhookDeliveryFormat;

/// New webhook subscription of the caller; private unless `visibility` says otherwise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionDto {
    #[serde(default)]
    pub visibility: Option<Visibility>,
    pub callback_address: String,
    pub topic_pattern: String,
    pub secret: Option<Secret<String>>,
    pub headers: Option<HashMap<String, String>>,
    pub retry_limit: Option<u32>,
    #[serde(default)]
    pub delivery_format: Option<WebhookDeliveryFormat>,
    pub expiration_time: Option<DateTime<Utc>>,
}

/// Partial subscription update; absent fields stay as they are.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSubscriptionDto {
    pub callback_address: Option<String>,
    pub topic_pattern: Option<String>,
    pub secret: Option<Secret<String>>,
    pub headers: Option<HashMap<String, String>>,
    pub retry_limit: Option<u32>,
    #[serde(default)]
    pub delivery_format: Option<WebhookDeliveryFormat>,
    pub active: Option<bool>,
    pub expiration_time: Option<DateTime<Utc>>,
}

impl UpdateSubscriptionDto {
    /// Takes the new secret; the `*****` a client echoes from a response keeps the current one.
    pub fn take_new_secret(&mut self) -> Option<Secret<String>> {
        self.secret.take().filter(|s| !s.is_mask())
    }
}

/// Event submitted through the HTTP API or forwarded by `RemoteBusSink`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishEventRequest {
    #[serde(default)]
    pub id: Option<Urn>,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub sequence: Option<u64>,
    #[serde(default)]
    pub visibility: Option<Visibility>,
    pub topic: String,
    #[serde(default)]
    pub event_type: Option<EventCategory>,
    #[serde(default)]
    pub severity: Option<EventSeverity>,
    #[serde(default)]
    pub outcome: Option<EventOutcome>,
    #[serde(default)]
    pub delivery_class: Option<DeliveryClass>,
    #[serde(default)]
    pub payload_type: Option<String>,
    #[serde(default)]
    pub source_crate: Option<String>,
    #[serde(default)]
    pub schema_version: Option<u32>,
    #[serde(default)]
    pub source: Option<EventSource>,
    #[serde(default)]
    pub actor: Option<EventActor>,
    #[serde(default)]
    pub subject: Option<EntityRef>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub causation_id: Option<String>,
    #[serde(default)]
    pub trace_context: Option<String>,
    #[serde(default)]
    pub flow_refs: Option<FlowRefs>,
    pub payload: serde_json::Value,
}

impl PublishEventRequest {
    /// Converts a canonical v2 envelope into an HTTP publish relay payload preserving identity.
    pub fn from_envelope(envelope: &EventEnvelope) -> Self {
        Self {
            id: Some(envelope.id.clone()),
            timestamp: Some(envelope.timestamp),
            sequence: envelope.sequence,
            visibility: Some(envelope.visibility_owner.visibility.clone()),
            topic: envelope.topic.as_str().to_string(),
            event_type: Some(envelope.event_type),
            severity: Some(envelope.severity),
            outcome: envelope.outcome,
            delivery_class: Some(envelope.delivery_class),
            payload_type: Some(envelope.payload_type.clone()),
            source_crate: Some(envelope.source.crate_name.clone()),
            schema_version: Some(envelope.schema_version),
            source: Some(envelope.source.clone()),
            actor: Some(envelope.actor.clone()),
            subject: envelope.subject.clone(),
            correlation_id: envelope.correlation.correlation_id.clone(),
            causation_id: envelope.correlation.causation_id.clone(),
            trace_context: envelope.correlation.trace_context.clone(),
            flow_refs: (!envelope.correlation.flow_refs.is_empty())
                .then(|| envelope.correlation.flow_refs.clone()),
            payload: envelope.payload.clone(),
        }
    }
}
