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

//! Immutable canonical v2 event envelope shared across storage, webhooks, OTLP and Kafka.

use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use opentelemetry::logs::Severity as OtelSeverity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use urn::Urn;
use uuid::Uuid;
use ymir::config::types::{EventSeverityLevel, NodeIdentitySnapshot};
use ymir::oauth::Owner;
use ymir::telemetry::TraceParent;
use ymir::types::oauth::UserTrait;

use crate::entities::topic::Topic;

static ENVELOPE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// High-level dataspace taxonomy of an event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCategory {
    #[default]
    AuditLifecycle,
    UsageControl,
    Attestation,
    DataAccess,
    ResourceChange,
    Security,
    Diagnostic,
}

impl EventCategory {
    /// Canonical PascalCase identifier stored in envelopes and indexes.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuditLifecycle => "AuditLifecycle",
            Self::UsageControl => "UsageControl",
            Self::Attestation => "Attestation",
            Self::DataAccess => "DataAccess",
            Self::ResourceChange => "ResourceChange",
            Self::Security => "Security",
            Self::Diagnostic => "Diagnostic",
        }
    }

    /// Lowercase kebab-case slug used for Kafka `ByCategory` topic routing.
    pub fn as_kebab_str(&self) -> &'static str {
        match self {
            Self::AuditLifecycle => "audit-lifecycle",
            Self::UsageControl => "usage-control",
            Self::Attestation => "attestation",
            Self::DataAccess => "data-access",
            Self::ResourceChange => "resource-change",
            Self::Security => "security",
            Self::Diagnostic => "diagnostic",
        }
    }
}

impl Display for EventCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EventCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auditlifecycle" | "audit_lifecycle" | "audit-lifecycle" => Ok(Self::AuditLifecycle),
            "usagecontrol" | "usage_control" | "usage-control" => Ok(Self::UsageControl),
            "attestation" => Ok(Self::Attestation),
            "dataaccess" | "data_access" | "data-access" => Ok(Self::DataAccess),
            "resourcechange" | "resource_change" | "resource-change" => Ok(Self::ResourceChange),
            "security" => Ok(Self::Security),
            "diagnostic" => Ok(Self::Diagnostic),
            other => Err(format!("unknown event category '{other}'")),
        }
    }
}

/// Ordered event severity aligned with `EventSeverityLevel` and OTel Logs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Critical,
}

impl EventSeverity {
    /// Canonical PascalCase severity string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "Debug",
            Self::Info => "Info",
            Self::Warn => "Warn",
            Self::Error => "Error",
            Self::Critical => "Critical",
        }
    }

    /// Maps into OpenTelemetry log severity numbers.
    pub fn to_otel_severity(&self) -> OtelSeverity {
        match self {
            Self::Debug => OtelSeverity::Debug,
            Self::Info => OtelSeverity::Info,
            Self::Warn => OtelSeverity::Warn,
            Self::Error => OtelSeverity::Error,
            Self::Critical => OtelSeverity::Fatal,
        }
    }

    /// Maps into OpenTelemetry uppercase severity text.
    pub fn as_otel_text(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Critical => "FATAL",
        }
    }
}

impl From<EventSeverityLevel> for EventSeverity {
    fn from(level: EventSeverityLevel) -> Self {
        match level {
            EventSeverityLevel::Debug => Self::Debug,
            EventSeverityLevel::Info => Self::Info,
            EventSeverityLevel::Warn => Self::Warn,
            EventSeverityLevel::Error => Self::Error,
            EventSeverityLevel::Critical => Self::Critical,
        }
    }
}

impl Display for EventSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EventSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            "critical" | "fatal" => Ok(Self::Critical),
            other => Err(format!("unknown event severity '{other}'")),
        }
    }
}

/// Outcome of the action reported by the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventOutcome {
    Success,
    Denied,
    Failed,
    Pending,
}

impl EventOutcome {
    /// Canonical PascalCase outcome string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Denied => "Denied",
            Self::Failed => "Failed",
            Self::Pending => "Pending",
        }
    }
}

impl Display for EventOutcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EventOutcome {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "success" => Ok(Self::Success),
            "denied" => Ok(Self::Denied),
            "failed" => Ok(Self::Failed),
            "pending" => Ok(Self::Pending),
            other => Err(format!("unknown event outcome '{other}'")),
        }
    }
}

/// Durability classification distinguishing audited business/governance events from high-volume telemetry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeliveryClass {
    #[default]
    AuditDurable,
    TelemetryStream,
}

impl DeliveryClass {
    /// Canonical PascalCase delivery class label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AuditDurable => "AuditDurable",
            Self::TelemetryStream => "TelemetryStream",
        }
    }
}

impl Display for DeliveryClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DeliveryClass {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auditdurable" | "audit_durable" | "audit-durable" => Ok(Self::AuditDurable),
            "telemetrystream" | "telemetry_stream" | "telemetry-stream" => {
                Ok(Self::TelemetryStream)
            }
            other => Err(format!("unknown delivery class '{other}'")),
        }
    }
}

/// Identity class of the actor that initiated the event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorType {
    HumanUser,
    PeerParticipant,
    AgentService,
    #[default]
    SystemWorker,
    Anonymous,
}

impl ActorType {
    /// Canonical PascalCase actor type label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HumanUser => "HumanUser",
            Self::PeerParticipant => "PeerParticipant",
            Self::AgentService => "AgentService",
            Self::SystemWorker => "SystemWorker",
            Self::Anonymous => "Anonymous",
        }
    }
}

impl Display for ActorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ActorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "humanuser" | "human_user" | "human-user" => Ok(Self::HumanUser),
            "peerparticipant" | "peer_participant" | "peer-participant" => {
                Ok(Self::PeerParticipant)
            }
            "agentservice" | "agent_service" | "agent-service" => Ok(Self::AgentService),
            "systemworker" | "system_worker" | "system-worker" => Ok(Self::SystemWorker),
            "anonymous" => Ok(Self::Anonymous),
            other => Err(format!("unknown actor type '{other}'")),
        }
    }
}

/// Origin node metadata stamped onto every canonical envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataspace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    pub service: String,
    pub instance_id: String,
    pub crate_name: String,
}

impl EventSource {
    /// Builds a placeholder source carrying only the emitting crate before pipeline normalization.
    pub fn for_crate(crate_name: impl Into<String>) -> Self {
        Self {
            dataspace_id: None,
            participant_id: None,
            service: "unknown".to_string(),
            instance_id: "unknown".to_string(),
            crate_name: crate_name.into(),
        }
    }

    /// Builds a fully populated source from a node identity snapshot.
    pub fn from_snapshot(snap: &NodeIdentitySnapshot, crate_name: impl Into<String>) -> Self {
        Self {
            dataspace_id: snap.dataspace_id.clone(),
            participant_id: snap.participant_id.clone(),
            service: snap.service.clone(),
            instance_id: snap.instance_id.clone(),
            crate_name: crate_name.into(),
        }
    }

    /// Fills unset node fields from `snap` without overwriting explicit origins.
    pub fn apply_snapshot(&mut self, snap: &NodeIdentitySnapshot) {
        if self.dataspace_id.is_none() {
            self.dataspace_id = snap.dataspace_id.clone();
        }
        if self.participant_id.is_none() {
            self.participant_id = snap.participant_id.clone();
        }
        if self.service.trim().is_empty() || self.service == "unknown" {
            self.service = snap.service.clone();
        }
        if self.instance_id.trim().is_empty() || self.instance_id == "unknown" {
            self.instance_id = snap.instance_id.clone();
        }
    }
}

/// Actor that triggered the action recorded by an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventActor {
    pub actor_type: ActorType,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl EventActor {
    /// Actor derived from an authenticated API user.
    pub fn human(user: &impl UserTrait) -> Self {
        Self {
            actor_type: ActorType::HumanUser,
            id: user.id().to_string(),
            participant_id: None,
            role: Some(user.role().to_string()),
        }
    }

    /// Actor representing a remote peer participant (DID / URN).
    pub fn peer(participant_id: impl Into<String>, role: Option<String>) -> Self {
        let did = participant_id.into();
        Self {
            actor_type: ActorType::PeerParticipant,
            id: did.clone(),
            participant_id: Some(did),
            role,
        }
    }

    /// Actor representing a service principal or microservice.
    pub fn agent(service_id: impl Into<String>, participant_id: Option<String>) -> Self {
        Self {
            actor_type: ActorType::AgentService,
            id: service_id.into(),
            participant_id,
            role: None,
        }
    }

    /// Internal background worker or boot actor.
    pub fn system() -> Self {
        Self {
            actor_type: ActorType::SystemWorker,
            id: "system".to_string(),
            participant_id: None,
            role: Some("/system".to_string()),
        }
    }

    /// Unauthenticated external request actor.
    pub fn anonymous() -> Self {
        Self {
            actor_type: ActorType::Anonymous,
            id: "anonymous".to_string(),
            participant_id: None,
            role: None,
        }
    }
}

/// Target entity reference for uniform cross-service indexing and UI routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

impl EntityRef {
    /// Creates a subject reference without a parent link.
    pub fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            parent_id: None,
        }
    }

    /// Attaches an optional parent identifier (such as an agreement under a transfer).
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }
}

/// Indexed cross-flow identifiers linking identity, negotiation, agreements and data access.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowRefs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agreement_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,
}

impl FlowRefs {
    /// Returns true when no flow reference fields are set.
    pub fn is_empty(&self) -> bool {
        self.agreement_id.is_none()
            && self.transfer_id.is_none()
            && self.negotiation_id.is_none()
            && self.grant_id.is_none()
            && self.dataset_id.is_none()
            && self.peer_id.is_none()
    }

    /// Picks the strongest partition key candidate across flow identifiers.
    pub fn primary_correlation(&self) -> Option<&str> {
        self.transfer_id
            .as_deref()
            .or(self.agreement_id.as_deref())
            .or(self.negotiation_id.as_deref())
            .or(self.grant_id.as_deref())
            .or(self.dataset_id.as_deref())
            .or(self.peer_id.as_deref())
    }

    /// Merges non-empty references from `other` when missing in `self`.
    pub fn merge_missing(&mut self, other: &Self) {
        if self.agreement_id.is_none() {
            self.agreement_id = other.agreement_id.clone();
        }
        if self.transfer_id.is_none() {
            self.transfer_id = other.transfer_id.clone();
        }
        if self.negotiation_id.is_none() {
            self.negotiation_id = other.negotiation_id.clone();
        }
        if self.grant_id.is_none() {
            self.grant_id = other.grant_id.clone();
        }
        if self.dataset_id.is_none() {
            self.dataset_id = other.dataset_id.clone();
        }
        if self.peer_id.is_none() {
            self.peer_id = other.peer_id.clone();
        }
    }
}

/// Causal and distributed tracing metadata attached to an event.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCorrelation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_context: Option<String>,
    #[serde(default)]
    pub flow_refs: FlowRefs,
}

/// Cryptographic signature metadata reserved for verifiable event attestation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventAttestation {
    pub format: String,
    pub signer_did: String,
    pub signature: String,
}

impl EventAttestation {
    /// Derives non-reversible SHA-256 attestation metadata from verified or issued credential bytes.
    pub fn from_material_digest(
        format: impl Into<String>,
        signer_did: impl Into<String>,
        signed_material: &str,
    ) -> Self {
        let hash = Sha256::digest(signed_material.as_bytes());
        let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
        Self {
            format: format.into(),
            signer_did: signer_did.into(),
            signature: format!("sha256:{hex}"),
        }
    }
}

/// Integrity digest over the canonicalized payload JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventIntegrity {
    pub payload_digest: String,
    pub canonicalization: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation: Option<EventAttestation>,
}

impl EventIntegrity {
    /// RFC 8785 canonicalization tag stored alongside SHA-256 digests.
    pub const RFC8785_SHA256: &'static str = "rfc8785-sha256";

    /// Computes a deterministic RFC 8785 SHA-256 digest over `payload`.
    pub fn for_payload(payload: &serde_json::Value) -> Self {
        let canonical = json_canon::to_string(payload).unwrap_or_else(|_| payload.to_string());
        let hash = Sha256::digest(canonical.as_bytes());
        let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
        Self {
            payload_digest: format!("sha256:{hex}"),
            canonicalization: Self::RFC8785_SHA256.to_string(),
            attestation: None,
        }
    }

    /// Returns true if `payload` matches `self.payload_digest` under RFC 8785 canonicalization.
    pub fn verify_payload(&self, payload: &serde_json::Value) -> bool {
        Self::for_payload(payload).payload_digest == self.payload_digest
    }
}

/// Canonical immutable v2 dataspace event envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Urn,
    pub spec_version: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,
    pub topic: Topic,
    pub event_type: EventCategory,
    pub severity: EventSeverity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<EventOutcome>,
    #[serde(default)]
    pub delivery_class: DeliveryClass,
    pub schema_version: u32,
    pub payload_type: String,
    pub source: EventSource,
    pub actor: EventActor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<EntityRef>,
    #[serde(default)]
    pub correlation: EventCorrelation,
    pub integrity: EventIntegrity,
    pub visibility_owner: Owner,
    pub payload: serde_json::Value,
}

/// Parameters for constructing an `EventEnvelope` with computed digest and defaults.
pub struct EventEnvelopeDraft {
    pub visibility_owner: Owner,
    pub topic: Topic,
    pub event_type: EventCategory,
    pub severity: EventSeverity,
    pub outcome: Option<EventOutcome>,
    pub delivery_class: DeliveryClass,
    pub schema_version: u32,
    pub payload_type: String,
    pub source: EventSource,
    pub actor: EventActor,
    pub subject: Option<EntityRef>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub trace_context: Option<String>,
    pub flow_refs: FlowRefs,
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Current canonical event envelope specification version.
    pub const SPEC_VERSION: &'static str = "2.0";

    /// Generates a fresh monotonic sequence number for process-local ordering.
    pub fn next_sequence() -> u64 {
        ENVELOPE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    }

    /// Generates a canonical `urn:uuid:<v4>` event identifier.
    pub fn new_id() -> Urn {
        let id_str = format!("urn:uuid:{}", Uuid::new_v4());
        Urn::from_str(&id_str).expect("valid urn:uuid format")
    }

    /// Builds an immutable v2 envelope from structured metadata and payload JSON.
    pub fn build(draft: EventEnvelopeDraft) -> Self {
        let trace_context = draft.trace_context.or_else(TraceParent::current);
        let integrity = EventIntegrity::for_payload(&draft.payload);
        let correlation_id = draft
            .correlation_id
            .or_else(|| draft.flow_refs.primary_correlation().map(ToOwned::to_owned))
            .or_else(|| draft.subject.as_ref().map(|s| s.id.clone()));

        Self {
            id: Self::new_id(),
            spec_version: Self::SPEC_VERSION.to_string(),
            timestamp: Utc::now(),
            sequence: Some(Self::next_sequence()),
            topic: draft.topic,
            event_type: draft.event_type,
            severity: draft.severity,
            outcome: draft.outcome,
            delivery_class: draft.delivery_class,
            schema_version: draft.schema_version,
            payload_type: draft.payload_type,
            source: draft.source,
            actor: draft.actor,
            subject: draft.subject,
            correlation: EventCorrelation {
                correlation_id,
                causation_id: draft.causation_id,
                trace_context,
                flow_refs: draft.flow_refs,
            },
            integrity,
            visibility_owner: draft.visibility_owner,
            payload: draft.payload,
        }
    }

    /// Convenience constructor for ad-hoc raw JSON envelopes (HTTP API and tests).
    pub fn new(
        visibility_owner: Owner,
        topic: Topic,
        crate_name: impl Into<String>,
        schema_version: u32,
        correlation_id: Option<String>,
        payload: serde_json::Value,
    ) -> Self {
        let crate_name = crate_name.into();
        let topic_name = topic.as_str().to_string();
        Self::build(EventEnvelopeDraft {
            visibility_owner,
            topic,
            event_type: EventCategory::AuditLifecycle,
            severity: EventSeverity::Info,
            outcome: Some(EventOutcome::Success),
            delivery_class: DeliveryClass::AuditDurable,
            schema_version,
            payload_type: format!("{crate_name}.{topic_name}.v{schema_version}"),
            source: EventSource::for_crate(crate_name),
            actor: EventActor::system(),
            subject: None,
            correlation_id,
            causation_id: None,
            trace_context: None,
            flow_refs: FlowRefs::default(),
            payload,
        })
    }

    /// Enriches unset origin node fields from the active process identity snapshot.
    pub fn apply_node_identity(&mut self, snap: &NodeIdentitySnapshot) {
        self.source.apply_snapshot(snap);
    }

    /// Attaches cryptographic attestation metadata to the envelope integrity block.
    pub fn with_attestation(mut self, attestation: EventAttestation) -> Self {
        self.integrity.attestation = Some(attestation);
        self
    }
}
