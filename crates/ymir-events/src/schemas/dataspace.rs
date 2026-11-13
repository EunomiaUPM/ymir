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

//! Canonical dataspace control-plane, dataplane, keystore and usage-control event payloads.

use serde::{Deserialize, Serialize};

use crate::entities::envelope::{
    DeliveryClass, EntityRef, EventCategory, EventOutcome, EventSeverity, FlowRefs,
};
use crate::entities::event::EventPayload;
use crate::entities::topic::Topic;

/// Coarse CRUD action for catalog, policy, connector and keystore resource changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAction {
    Created,
    Updated,
    Deleted,
    Imported,
}

impl ResourceAction {
    /// Lowercase action verb appended to resource topics.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Deleted => "deleted",
            Self::Imported => "imported",
        }
    }
}

/// Compact catalog resource lifecycle payload (never dumps full JSON-LD graphs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogResourcePayload {
    pub action: ResourceAction,
    /// Resource kind segment (`datasets`, `catalogs`, `schemas`, `policy_templates`).
    pub resource_kind: String,
    pub resource_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

impl EventPayload for CatalogResourcePayload {
    const TOPIC: &'static str = "catalog:updated";
    const PAYLOAD_TYPE: &'static str = "dataspace.catalog_resource.v1";
    const CATEGORY: EventCategory = EventCategory::ResourceChange;
    const CRATE_NAME: &'static str = "catalog_agent";

    fn topic(&self) -> Topic {
        Topic::new(format!("{}:{}", self.resource_kind, self.action.as_str()))
            .unwrap_or_else(|_| Topic::new(Self::TOPIC).expect("valid static fallback topic"))
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new(&self.resource_kind, &self.resource_id))
    }

    fn flow_refs(&self) -> FlowRefs {
        let mut refs = FlowRefs::default();
        if self.resource_kind == "datasets" || self.resource_kind == "resources" {
            refs.dataset_id = Some(self.resource_id.clone());
        }
        refs
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.resource_id.clone())
    }
}

/// Redacted ODRL policy summary emitted by catalog/policy services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDefinitionPayload {
    pub action: ResourceAction,
    pub policy_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub permission_count: usize,
    pub prohibition_count: usize,
    pub obligation_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_digest: Option<String>,
}

impl EventPayload for PolicyDefinitionPayload {
    const TOPIC: &'static str = "policies:updated";
    const PAYLOAD_TYPE: &'static str = "dataspace.policy_definition.v1";
    const CATEGORY: EventCategory = EventCategory::ResourceChange;
    const CRATE_NAME: &'static str = "catalog_agent";

    fn topic(&self) -> Topic {
        Topic::new(format!("policies:{}", self.action.as_str())).expect("valid static policy topic")
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new("policy", &self.policy_id))
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.policy_id.clone())
    }
}

/// Semantic contract negotiation state transition payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiationTransitionPayload {
    /// Specific transition segment (`requested`, `offered`, `accepted`, `agreed`, `verified`, `finalized`, `terminated`, `updated`).
    pub transition: String,
    pub negotiation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agreement_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<String>,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_party_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_party_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
}

impl EventPayload for NegotiationTransitionPayload {
    const TOPIC: &'static str = "negotiations:updated";
    const PAYLOAD_TYPE: &'static str = "dataspace.negotiation_transition.v1";
    const CATEGORY: EventCategory = EventCategory::AuditLifecycle;
    const CRATE_NAME: &'static str = "negotiation_agent";

    fn topic(&self) -> Topic {
        Topic::new(format!("negotiations:{}", self.transition))
            .unwrap_or_else(|_| Topic::new(Self::TOPIC).expect("valid static negotiation topic"))
    }

    fn severity(&self) -> EventSeverity {
        if self.transition == "terminated" || self.error_reason.is_some() {
            EventSeverity::Warn
        } else {
            EventSeverity::Info
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(match self.transition.as_str() {
            "terminated" if self.error_reason.is_some() => EventOutcome::Failed,
            "terminated" => EventOutcome::Denied,
            "requested" | "offered" => EventOutcome::Pending,
            _ => EventOutcome::Success,
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        let mut reference = EntityRef::new("negotiation_process", &self.negotiation_id);
        if let Some(agreement_id) = &self.agreement_id {
            reference = reference.with_parent(agreement_id);
        }
        Some(reference)
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            agreement_id: self.agreement_id.clone(),
            negotiation_id: Some(self.negotiation_id.clone()),
            dataset_id: self.dataset_id.clone(),
            peer_id: self.counter_party_id.clone(),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.negotiation_id.clone())
    }
}

/// Agreement lifecycle event carrying policy integrity metadata for downstream usage control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgreementLifecyclePayload {
    pub action: String,
    pub agreement_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consumer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_digest: Option<String>,
    pub state: String,
}

impl EventPayload for AgreementLifecyclePayload {
    const TOPIC: &'static str = "agreements:updated";
    const PAYLOAD_TYPE: &'static str = "dataspace.agreement_lifecycle.v1";
    const CATEGORY: EventCategory = EventCategory::AuditLifecycle;
    const CRATE_NAME: &'static str = "negotiation_agent";

    fn topic(&self) -> Topic {
        Topic::new(format!("agreements:{}", self.action))
            .unwrap_or_else(|_| Topic::new(Self::TOPIC).expect("valid static agreement topic"))
    }

    fn subject(&self) -> Option<EntityRef> {
        let mut reference = EntityRef::new("agreement", &self.agreement_id);
        if let Some(negotiation_id) = &self.negotiation_id {
            reference = reference.with_parent(negotiation_id);
        }
        Some(reference)
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            agreement_id: Some(self.agreement_id.clone()),
            negotiation_id: self.negotiation_id.clone(),
            dataset_id: self.dataset_id.clone(),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.agreement_id.clone())
    }
}

/// Control-plane data transfer state transition payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferTransitionPayload {
    /// Specific transition segment (`requested`, `started`, `suspended`, `completed`, `terminated`, `updated`).
    pub transition: String,
    pub transfer_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agreement_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<String>,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_type: Option<String>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_party_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
}

impl EventPayload for TransferTransitionPayload {
    const TOPIC: &'static str = "transfers:updated";
    const PAYLOAD_TYPE: &'static str = "dataspace.transfer_transition.v1";
    const CATEGORY: EventCategory = EventCategory::AuditLifecycle;
    const CRATE_NAME: &'static str = "transfer_agent";

    fn topic(&self) -> Topic {
        Topic::new(format!("transfers:{}", self.transition))
            .unwrap_or_else(|_| Topic::new(Self::TOPIC).expect("valid static transfer topic"))
    }

    fn severity(&self) -> EventSeverity {
        if self.transition == "terminated" || self.error_reason.is_some() {
            EventSeverity::Warn
        } else {
            EventSeverity::Info
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(match self.transition.as_str() {
            "terminated" if self.error_reason.is_some() => EventOutcome::Failed,
            "requested" | "suspended" => EventOutcome::Pending,
            _ => EventOutcome::Success,
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        let mut reference = EntityRef::new("transfer_process", &self.transfer_id);
        if let Some(agreement_id) = &self.agreement_id {
            reference = reference.with_parent(agreement_id);
        }
        Some(reference)
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            agreement_id: self.agreement_id.clone(),
            transfer_id: Some(self.transfer_id.clone()),
            dataset_id: self.dataset_id.clone(),
            peer_id: self.counter_party_id.clone(),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.transfer_id.clone())
    }
}

/// Direction of a DSP or dataspace protocol exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    Incoming,
    Outgoing,
}

/// DSP or inter-agent protocol message exchange metadata (without large full payloads).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolMessagePayload {
    pub direction: MessageDirection,
    pub protocol: String,
    pub message_type: String,
    pub process_kind: String,
    pub process_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_address: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
}

impl EventPayload for ProtocolMessagePayload {
    const TOPIC: &'static str = "messages:exchanged";
    const PAYLOAD_TYPE: &'static str = "dataspace.protocol_message.v1";
    const CATEGORY: EventCategory = EventCategory::AuditLifecycle;
    const CRATE_NAME: &'static str = "dsp";

    fn topic(&self) -> Topic {
        let verb = match self.direction {
            MessageDirection::Incoming => "received",
            MessageDirection::Outgoing => "sent",
        };
        Topic::new(format!("{}_messages:{verb}", self.process_kind))
            .unwrap_or_else(|_| Topic::new(Self::TOPIC).expect("valid static message topic"))
    }

    fn severity(&self) -> EventSeverity {
        if self.error_reason.is_some() {
            EventSeverity::Error
        } else {
            EventSeverity::Info
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(if self.error_reason.is_some() {
            EventOutcome::Failed
        } else {
            EventOutcome::Success
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new(&self.process_kind, &self.process_id))
    }

    fn flow_refs(&self) -> FlowRefs {
        let mut refs = FlowRefs {
            peer_id: self.peer_id.clone(),
            ..FlowRefs::default()
        };
        if self.process_kind.contains("negotiation") {
            refs.negotiation_id = Some(self.process_id.clone());
        } else if self.process_kind.contains("transfer") {
            refs.transfer_id = Some(self.process_id.clone());
        }
        refs
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.process_id.clone())
    }
}

/// Connector instance or template lifecycle change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorLifecyclePayload {
    pub action: ResourceAction,
    /// `instance` or `template`.
    pub connector_kind: String,
    pub connector_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connector_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl EventPayload for ConnectorLifecyclePayload {
    const TOPIC: &'static str = "connectors:updated";
    const PAYLOAD_TYPE: &'static str = "dataspace.connector.v1";
    const CATEGORY: EventCategory = EventCategory::ResourceChange;
    const CRATE_NAME: &'static str = "connector";

    fn topic(&self) -> Topic {
        Topic::new(format!("connector_{}s:{}", self.connector_kind, self.action.as_str()))
            .unwrap_or_else(|_| Topic::new(Self::TOPIC).expect("valid static connector topic"))
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new(
            format!("connector_{}", self.connector_kind),
            &self.connector_id,
        ))
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.connector_id.clone())
    }
}

/// Keystore security audit event — enforces zero secret material in emitted payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeystoreAuditPayload {
    pub action: ResourceAction,
    /// `secret` or `parameter`.
    pub entry_kind: String,
    pub entry_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_type: Option<String>,
    pub value_redacted: bool,
}

impl EventPayload for KeystoreAuditPayload {
    const TOPIC: &'static str = "keystore:updated";
    const PAYLOAD_TYPE: &'static str = "security.keystore_audit.v1";
    const CATEGORY: EventCategory = EventCategory::Security;
    const CRATE_NAME: &'static str = "keystore";

    fn topic(&self) -> Topic {
        Topic::new(format!("keystore_{}s:{}", self.entry_kind, self.action.as_str()))
            .unwrap_or_else(|_| Topic::new(Self::TOPIC).expect("valid static keystore topic"))
    }

    fn severity(&self) -> EventSeverity {
        match self.action {
            ResourceAction::Deleted => EventSeverity::Warn,
            _ => EventSeverity::Info,
        }
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new(&self.entry_kind, &self.entry_id))
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.entry_id.clone())
    }
}

/// Dataplane execution state transition (replaces isolated `transfer_events` table writes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataplaneStatePayload {
    pub transfer_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agreement_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<String>,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<String>,
    pub transfer_type: String,
    pub direction: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
}

impl EventPayload for DataplaneStatePayload {
    const TOPIC: &'static str = "dataplane:transfer:state_changed";
    const PAYLOAD_TYPE: &'static str = "dataplane.transfer_state.v1";
    const CATEGORY: EventCategory = EventCategory::AuditLifecycle;
    const CRATE_NAME: &'static str = "dataplane";

    fn severity(&self) -> EventSeverity {
        if self.error_reason.is_some() {
            EventSeverity::Error
        } else {
            EventSeverity::Info
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(if self.error_reason.is_some() {
            EventOutcome::Failed
        } else {
            EventOutcome::Success
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        let mut reference = EntityRef::new("transfer_process", &self.transfer_id);
        if let Some(agreement_id) = &self.agreement_id {
            reference = reference.with_parent(agreement_id);
        }
        Some(reference)
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            agreement_id: self.agreement_id.clone(),
            transfer_id: Some(self.transfer_id.clone()),
            dataset_id: self.dataset_id.clone(),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.transfer_id.clone())
    }
}

/// High-frequency dataplane data access / proxy / relay telemetry record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataAccessPayload {
    pub allowed: bool,
    pub transfer_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agreement_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<String>,
    /// Channel (`http_proxy` or `kafka_relay`).
    pub channel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_transferred: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denial_reason: Option<String>,
}

impl EventPayload for DataAccessPayload {
    const TOPIC: &'static str = "dataplane:data:accessed";
    const PAYLOAD_TYPE: &'static str = "dataplane.data_access.v1";
    const CATEGORY: EventCategory = EventCategory::DataAccess;
    const CRATE_NAME: &'static str = "dataplane";

    fn topic(&self) -> Topic {
        let name = if self.allowed {
            "dataplane:data:accessed"
        } else {
            "dataplane:data:rejected"
        };
        Topic::new(name).expect("valid static data access topic")
    }

    fn delivery_class(&self) -> DeliveryClass {
        DeliveryClass::TelemetryStream
    }

    fn severity(&self) -> EventSeverity {
        if self.allowed {
            EventSeverity::Info
        } else {
            EventSeverity::Warn
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(if self.allowed {
            EventOutcome::Success
        } else {
            EventOutcome::Denied
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        let mut reference = EntityRef::new("transfer_process", &self.transfer_id);
        if let Some(agreement_id) = &self.agreement_id {
            reference = reference.with_parent(agreement_id);
        }
        Some(reference)
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            agreement_id: self.agreement_id.clone(),
            transfer_id: Some(self.transfer_id.clone()),
            dataset_id: self.dataset_id.clone(),
            peer_id: self.peer_id.clone(),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.transfer_id.clone())
    }
}

/// Usage-control PDP authorization result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecisionOutcome {
    Permit,
    Deny,
}

/// Usage-control policy decision point evaluation audit payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDecisionPayload {
    pub decision: PolicyDecisionOutcome,
    /// Enforcement point (`catalog`, `negotiation`, `transfer`, `dataplane_http`, `dataplane_kafka`).
    pub enforcement_point: String,
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agreement_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,
    #[serde(default)]
    pub obligations_triggered: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl EventPayload for PolicyDecisionPayload {
    const TOPIC: &'static str = "usage_control:pdp:evaluated";
    const PAYLOAD_TYPE: &'static str = "usage_control.decision.v1";
    const CATEGORY: EventCategory = EventCategory::UsageControl;
    const CRATE_NAME: &'static str = "usage_control";

    fn severity(&self) -> EventSeverity {
        match self.decision {
            PolicyDecisionOutcome::Permit => EventSeverity::Info,
            PolicyDecisionOutcome::Deny => EventSeverity::Warn,
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(match self.decision {
            PolicyDecisionOutcome::Permit => EventOutcome::Success,
            PolicyDecisionOutcome::Deny => EventOutcome::Denied,
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        if let Some(transfer_id) = &self.transfer_id {
            return Some(EntityRef::new("transfer_process", transfer_id));
        }
        if let Some(agreement_id) = &self.agreement_id {
            return Some(EntityRef::new("agreement", agreement_id));
        }
        self.policy_id.as_deref().map(|id| EntityRef::new("policy", id))
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            agreement_id: self.agreement_id.clone(),
            transfer_id: self.transfer_id.clone(),
            dataset_id: self.dataset_id.clone(),
            peer_id: self.peer_id.clone(),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        self.transfer_id
            .clone()
            .or_else(|| self.agreement_id.clone())
            .or_else(|| self.policy_id.clone())
    }
}

/// Usage-control obligation / WASM transformer execution audit payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObligationExecutionPayload {
    pub obligation_id: String,
    pub obligation_kind: String,
    pub enforcement_point: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agreement_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_id: Option<String>,
    pub succeeded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
}

impl EventPayload for ObligationExecutionPayload {
    const TOPIC: &'static str = "usage_control:obligation:executed";
    const PAYLOAD_TYPE: &'static str = "usage_control.obligation.v1";
    const CATEGORY: EventCategory = EventCategory::UsageControl;
    const CRATE_NAME: &'static str = "usage_control";

    fn severity(&self) -> EventSeverity {
        if self.succeeded {
            EventSeverity::Info
        } else {
            EventSeverity::Error
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(if self.succeeded {
            EventOutcome::Success
        } else {
            EventOutcome::Failed
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new("obligation", &self.obligation_id))
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            agreement_id: self.agreement_id.clone(),
            transfer_id: self.transfer_id.clone(),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        self.transfer_id
            .clone()
            .or_else(|| self.agreement_id.clone())
            .or_else(|| Some(self.obligation_id.clone()))
    }
}
