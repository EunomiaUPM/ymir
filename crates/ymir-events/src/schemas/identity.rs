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

//! Canonical identity, GNAP grant, VC attestation, interaction and registry payloads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::entities::envelope::{EntityRef, EventCategory, EventOutcome, EventSeverity, FlowRefs};
use crate::entities::event::EventPayload;
use crate::entities::topic::Topic;

/// Direction of a GNAP identity grant relative to the emitting participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantDirection {
    Received,
    Requested,
}

/// Lifecycle transition of a GNAP identity grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantAction {
    Created,
    Approved,
    Finalized,
    Rejected,
    Revoked,
    Updated,
}

impl GrantAction {
    /// Emitted event topic for this grant lifecycle action.
    pub fn topic(&self) -> &'static str {
        match self {
            Self::Created => "grants:created",
            Self::Approved => "grants:approved",
            Self::Finalized => "grants:finalized",
            Self::Rejected => "grants:rejected",
            Self::Revoked => "grants:revoked",
            Self::Updated => "grants:updated",
        }
    }
}

/// Redacted GNAP grant lifecycle payload shared by Heimdall and consumer/provider `auth`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityGrantPayload {
    pub action: GrantAction,
    pub grant_id: String,
    pub direction: GrantDirection,
    pub kind: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_participant_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_participant_nick: Option<String>,
    #[serde(default)]
    pub vc_types: Vec<String>,
    /// Short SHA-256 fingerprint (`sha256:xxxxxxxxxxxx`) when an access token is issued; never raw.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl IdentityGrantPayload {
    /// Computes a short non-reversible SHA-256 prefix for correlation without leaking tokens.
    pub fn fingerprint_token(token: &str) -> String {
        let digest = Sha256::digest(token.as_bytes());
        let prefix: String = digest.iter().take(8).map(|b| format!("{b:02x}")).collect();
        format!("sha256:{prefix}")
    }
}

impl EventPayload for IdentityGrantPayload {
    const TOPIC: &'static str = "grants:updated";
    const PAYLOAD_TYPE: &'static str = "identity.grant.v1";
    const CATEGORY: EventCategory = EventCategory::AuditLifecycle;
    const CRATE_NAME: &'static str = "identity";

    fn topic(&self) -> Topic {
        Topic::new(self.action.topic()).expect("valid static grant topic")
    }

    fn severity(&self) -> EventSeverity {
        match self.action {
            GrantAction::Rejected | GrantAction::Revoked => EventSeverity::Warn,
            _ => EventSeverity::Info,
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(match self.action {
            GrantAction::Rejected => EventOutcome::Denied,
            GrantAction::Created => EventOutcome::Pending,
            _ => EventOutcome::Success,
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new("grant", &self.grant_id))
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            grant_id: Some(self.grant_id.clone()),
            peer_id: self.peer_participant_id.clone(),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.grant_id.clone())
    }
}

/// Direction of an OID4VP / DID presentation verification flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationDirection {
    Incoming,
    Outgoing,
}

/// Lifecycle transition of a verifiable presentation verification challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationAction {
    Requested,
    Verified,
    Failed,
    Updated,
}

impl VerificationAction {
    /// Emitted event topic for this verification step.
    pub fn topic(&self) -> &'static str {
        match self {
            Self::Requested => "verifications:requested",
            Self::Verified => "verifications:verified",
            Self::Failed => "verifications:failed",
            Self::Updated => "verifications:updated",
        }
    }
}

/// Verifiable presentation attestation lifecycle payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationPayload {
    pub action: VerificationAction,
    pub grant_id: String,
    pub verification_id: String,
    pub direction: VerificationDirection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holder_did: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier_did: Option<String>,
    #[serde(default)]
    pub vc_types: Vec<String>,
    #[serde(default)]
    pub presented_vc_ids: Vec<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

impl EventPayload for VerificationPayload {
    const TOPIC: &'static str = "verifications:updated";
    const PAYLOAD_TYPE: &'static str = "identity.verification.v1";
    const CATEGORY: EventCategory = EventCategory::Attestation;
    const CRATE_NAME: &'static str = "identity";

    fn topic(&self) -> Topic {
        Topic::new(self.action.topic()).expect("valid static verification topic")
    }

    fn severity(&self) -> EventSeverity {
        match self.action {
            VerificationAction::Failed => EventSeverity::Warn,
            _ => EventSeverity::Info,
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(match self.action {
            VerificationAction::Failed => EventOutcome::Failed,
            VerificationAction::Requested => EventOutcome::Pending,
            VerificationAction::Verified | VerificationAction::Updated => EventOutcome::Success,
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new("verification", &self.verification_id).with_parent(&self.grant_id))
    }

    fn flow_refs(&self) -> FlowRefs {
        let peer_id = match self.direction {
            VerificationDirection::Incoming => self.holder_did.clone(),
            VerificationDirection::Outgoing => self.verifier_did.clone(),
        };
        FlowRefs {
            grant_id: Some(self.grant_id.clone()),
            peer_id,
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.grant_id.clone())
    }
}

/// Lifecycle transition of OID4VCI credential issuance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuanceAction {
    Initiated,
    Issued,
    Failed,
    Updated,
}

impl IssuanceAction {
    /// Emitted event topic for this issuance step.
    pub fn topic(&self) -> &'static str {
        match self {
            Self::Initiated => "issuances:initiated",
            Self::Issued => "issuances:issued",
            Self::Failed => "issuances:failed",
            Self::Updated => "issuances:updated",
        }
    }
}

/// Verifiable Credential issuance attestation payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialIssuancePayload {
    pub action: IssuanceAction,
    pub grant_id: String,
    pub credential_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer_did: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holder_did: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_name: Option<String>,
    #[serde(default)]
    pub vc_types: Vec<String>,
    pub issued: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

impl EventPayload for CredentialIssuancePayload {
    const TOPIC: &'static str = "issuances:updated";
    const PAYLOAD_TYPE: &'static str = "identity.credential_issuance.v1";
    const CATEGORY: EventCategory = EventCategory::Attestation;
    const CRATE_NAME: &'static str = "identity";

    fn topic(&self) -> Topic {
        Topic::new(self.action.topic()).expect("valid static issuance topic")
    }

    fn severity(&self) -> EventSeverity {
        match self.action {
            IssuanceAction::Failed => EventSeverity::Error,
            _ => EventSeverity::Info,
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(match self.action {
            IssuanceAction::Failed => EventOutcome::Failed,
            IssuanceAction::Initiated => EventOutcome::Pending,
            IssuanceAction::Issued | IssuanceAction::Updated => EventOutcome::Success,
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new("issuance", &self.credential_id).with_parent(&self.grant_id))
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            grant_id: Some(self.grant_id.clone()),
            peer_id: self.holder_did.clone().or_else(|| self.issuer_did.clone()),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.grant_id.clone())
    }
}

/// Lifecycle transition of a user/wallet interaction challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionAction {
    Started,
    Completed,
    Failed,
    Updated,
}

impl InteractionAction {
    /// Emitted event topic for this interaction step.
    pub fn topic(&self) -> &'static str {
        match self {
            Self::Started => "interactions:started",
            Self::Completed => "interactions:completed",
            Self::Failed => "interactions:failed",
            Self::Updated => "interactions:updated",
        }
    }
}

/// GNAP/wallet interaction challenge event payload (callback tokens omitted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionPayload {
    pub action: InteractionAction,
    pub grant_id: String,
    pub interaction_id: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_uri: Option<String>,
    pub outcome_status: String,
}

impl EventPayload for InteractionPayload {
    const TOPIC: &'static str = "interactions:updated";
    const PAYLOAD_TYPE: &'static str = "identity.interaction.v1";
    const CATEGORY: EventCategory = EventCategory::AuditLifecycle;
    const CRATE_NAME: &'static str = "identity";

    fn topic(&self) -> Topic {
        Topic::new(self.action.topic()).expect("valid static interaction topic")
    }

    fn severity(&self) -> EventSeverity {
        match self.action {
            InteractionAction::Failed => EventSeverity::Warn,
            _ => EventSeverity::Info,
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(match self.action {
            InteractionAction::Failed => EventOutcome::Failed,
            InteractionAction::Started => EventOutcome::Pending,
            InteractionAction::Completed | InteractionAction::Updated => EventOutcome::Success,
        })
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new("interaction", &self.interaction_id).with_parent(&self.grant_id))
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            grant_id: Some(self.grant_id.clone()),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.grant_id.clone())
    }
}

/// Lifecycle transition of a dataspace participant registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantAction {
    Registered,
    Updated,
    TrustUpdated,
    Revoked,
    Removed,
}

impl ParticipantAction {
    /// Emitted event topic for this participant registry action.
    pub fn topic(&self) -> &'static str {
        match self {
            Self::Registered => "participants:registered",
            Self::Updated => "participants:updated",
            Self::TrustUpdated => "participants:trust_updated",
            Self::Revoked => "participants:revoked",
            Self::Removed => "participants:removed",
        }
    }
}

/// Redacted participant trust and onboarding registry payload (never exposes raw DB models).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantRegistryPayload {
    pub action: ParticipantAction,
    pub participant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_nick: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub participant_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl EventPayload for ParticipantRegistryPayload {
    const TOPIC: &'static str = "participants:updated";
    const PAYLOAD_TYPE: &'static str = "identity.participant.v1";
    const CATEGORY: EventCategory = EventCategory::AuditLifecycle;
    const CRATE_NAME: &'static str = "identity";

    fn topic(&self) -> Topic {
        Topic::new(self.action.topic()).expect("valid static participant topic")
    }

    fn severity(&self) -> EventSeverity {
        match self.action {
            ParticipantAction::Revoked => EventSeverity::Warn,
            _ => EventSeverity::Info,
        }
    }

    fn outcome(&self) -> Option<EventOutcome> {
        Some(EventOutcome::Success)
    }

    fn subject(&self) -> Option<EntityRef> {
        Some(EntityRef::new("participant", &self.participant_id))
    }

    fn flow_refs(&self) -> FlowRefs {
        FlowRefs {
            peer_id: Some(self.participant_id.clone()),
            ..FlowRefs::default()
        }
    }

    fn correlation_id(&self) -> Option<String> {
        Some(self.participant_id.clone())
    }
}
