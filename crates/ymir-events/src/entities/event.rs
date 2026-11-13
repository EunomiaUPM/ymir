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

//! Typed domain payload trait (`EventPayload`) and call-site emission context (`EventContext`).

use serde::Serialize;
use ymir::errors::{Errors, Outcome};
use ymir::oauth::Owner;
use ymir::telemetry::TraceParent;
use ymir::types::oauth::UserTrait;
use ymir::types::participants::Visibility;

use crate::entities::envelope::{
    DeliveryClass, EntityRef, EventActor, EventAttestation, EventCategory, EventEnvelope,
    EventEnvelopeDraft, EventOutcome, EventSeverity, EventSource, FlowRefs,
};
use crate::entities::topic::Topic;

/// Use-case-specific event payload embedded cleanly inside a canonical [`EventEnvelope`].
pub trait EventPayload: Serialize + Send + Sync + 'static {
    /// Default topic when a payload covers one action (override [`EventPayload::topic`] otherwise).
    const TOPIC: &'static str;
    /// Stable versioned payload schema identifier (e.g. `identity.grant.v1`).
    const PAYLOAD_TYPE: &'static str;
    /// High-level dataspace taxonomy.
    const CATEGORY: EventCategory;
    /// Emitting library or service crate name.
    const CRATE_NAME: &'static str;
    /// Monotonic payload schema revision.
    const SCHEMA_VERSION: u32 = 1;

    /// Concrete event topic for this payload instance.
    fn topic(&self) -> Topic {
        Topic::new(Self::TOPIC).expect("valid static event topic")
    }

    /// Persistence durability class (`AuditDurable` by default).
    fn delivery_class(&self) -> DeliveryClass {
        DeliveryClass::AuditDurable
    }

    /// Severity of the reported action.
    fn severity(&self) -> EventSeverity {
        EventSeverity::Info
    }

    /// Execution outcome of the reported action.
    fn outcome(&self) -> Option<EventOutcome> {
        Some(EventOutcome::Success)
    }

    /// Primary entity subject referenced by this event.
    fn subject(&self) -> Option<EntityRef> {
        None
    }

    /// Cross-flow correlation references derivable directly from the payload.
    fn flow_refs(&self) -> FlowRefs {
        FlowRefs::default()
    }

    /// Preferred flow correlation identifier (defaults to `flow_refs` or `subject`).
    fn correlation_id(&self) -> Option<String> {
        None
    }

    /// Materializes this payload and emission context into a canonical [`EventEnvelope`].
    fn into_envelope(&self, ctx: &EventContext, source: EventSource) -> Outcome<EventEnvelope> {
        let payload =
            serde_json::to_value(self).map_err(|e| Errors::parse(e.to_string(), None))?;
        let mut flow_refs = ctx.flow_refs.clone();
        flow_refs.merge_missing(&self.flow_refs());
        let correlation_id = ctx
            .correlation_id
            .clone()
            .or_else(|| self.correlation_id())
            .or_else(|| flow_refs.primary_correlation().map(ToOwned::to_owned));

        let mut envelope = EventEnvelope::build(EventEnvelopeDraft {
            visibility_owner: ctx.visibility_owner.clone(),
            topic: self.topic(),
            event_type: Self::CATEGORY,
            severity: self.severity(),
            outcome: self.outcome(),
            delivery_class: self.delivery_class(),
            schema_version: Self::SCHEMA_VERSION,
            payload_type: Self::PAYLOAD_TYPE.to_string(),
            source,
            actor: ctx.actor.clone(),
            subject: self.subject(),
            correlation_id,
            causation_id: ctx.causation_id.clone(),
            trace_context: ctx.trace_context.clone(),
            flow_refs,
            payload,
        });
        if let Some(attestation) = &ctx.attestation {
            envelope = envelope.with_attestation(attestation.clone());
        }
        Ok(envelope)
    }
}

/// Call-site metadata describing who performed an action, visibility scope and causal trace.
#[derive(Debug, Clone)]
pub struct EventContext {
    pub visibility_owner: Owner,
    pub actor: EventActor,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub trace_context: Option<String>,
    pub flow_refs: FlowRefs,
    pub attestation: Option<EventAttestation>,
}

impl EventContext {
    /// Context for an interactive API caller acting on their private record.
    pub fn for_user(user: &impl UserTrait) -> Self {
        Self {
            visibility_owner: Owner::private(user),
            actor: EventActor::human(user),
            correlation_id: None,
            causation_id: None,
            trace_context: TraceParent::current(),
            flow_refs: FlowRefs::default(),
            attestation: None,
        }
    }

    /// Context for a user action creating a record with explicit visibility.
    pub fn for_user_with_visibility(user: &impl UserTrait, visibility: Visibility) -> Self {
        Self {
            visibility_owner: Owner::of(user, Some(visibility)),
            actor: EventActor::human(user),
            correlation_id: None,
            causation_id: None,
            trace_context: TraceParent::current(),
            flow_refs: FlowRefs::default(),
            attestation: None,
        }
    }

    /// Context for a background or service-owned record (`SystemWorker` actor by default).
    pub fn for_owner(visibility_owner: Owner) -> Self {
        Self {
            visibility_owner,
            actor: EventActor::system(),
            correlation_id: None,
            causation_id: None,
            trace_context: TraceParent::current(),
            flow_refs: FlowRefs::default(),
            attestation: None,
        }
    }

    /// Context for an incoming peer protocol message targeting a local `visibility_owner`.
    pub fn for_peer(
        visibility_owner: Owner,
        peer_id: impl Into<String>,
        role: Option<String>,
    ) -> Self {
        let peer_id = peer_id.into();
        let mut flow_refs = FlowRefs::default();
        flow_refs.peer_id = Some(peer_id.clone());
        Self {
            visibility_owner,
            actor: EventActor::peer(peer_id, role),
            correlation_id: None,
            causation_id: None,
            trace_context: TraceParent::current(),
            flow_refs,
            attestation: None,
        }
    }

    /// Context for system/boot activity scoped to an owner.
    pub fn system(visibility_owner: Owner) -> Self {
        Self::for_owner(visibility_owner)
    }

    /// Updates record visibility while preserving user and role ownership.
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility_owner.visibility = visibility;
        self
    }

    /// Overrides the actor responsible for the event.
    pub fn with_actor(mut self, actor: EventActor) -> Self {
        self.actor = actor;
        self
    }

    /// Attaches a flow correlation identifier (e.g. `grant_id`, agreement or transfer ID).
    pub fn with_correlation(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Attaches the upstream causing event/message identifier.
    pub fn with_causation(mut self, causation_id: impl Into<String>) -> Self {
        self.causation_id = Some(causation_id.into());
        self
    }

    /// Attaches an explicit W3C `traceparent` string when resuming across async boundaries.
    pub fn with_trace_context(mut self, trace_context: Option<String>) -> Self {
        self.trace_context = trace_context;
        self
    }

    /// Replaces or sets cross-flow references.
    pub fn with_flow_refs(mut self, flow_refs: FlowRefs) -> Self {
        self.flow_refs = flow_refs;
        self
    }

    /// Attaches cryptographic attestation metadata to the resulting envelope.
    pub fn with_attestation(mut self, attestation: EventAttestation) -> Self {
        self.attestation = Some(attestation);
        self
    }

    /// Sets the agreement reference in `flow_refs`.
    pub fn with_agreement_id(mut self, id: impl Into<String>) -> Self {
        self.flow_refs.agreement_id = Some(id.into());
        self
    }

    /// Sets the transfer process reference in `flow_refs`.
    pub fn with_transfer_id(mut self, id: impl Into<String>) -> Self {
        self.flow_refs.transfer_id = Some(id.into());
        self
    }

    /// Sets the negotiation process reference in `flow_refs`.
    pub fn with_negotiation_id(mut self, id: impl Into<String>) -> Self {
        self.flow_refs.negotiation_id = Some(id.into());
        self
    }

    /// Sets the identity grant reference in `flow_refs`.
    pub fn with_grant_id(mut self, id: impl Into<String>) -> Self {
        self.flow_refs.grant_id = Some(id.into());
        self
    }

    /// Sets the catalog dataset reference in `flow_refs`.
    pub fn with_dataset_id(mut self, id: impl Into<String>) -> Self {
        self.flow_refs.dataset_id = Some(id.into());
        self
    }

    /// Sets the remote peer identifier in `flow_refs`.
    pub fn with_peer_id(mut self, id: impl Into<String>) -> Self {
        self.flow_refs.peer_id = Some(id.into());
        self
    }
}
