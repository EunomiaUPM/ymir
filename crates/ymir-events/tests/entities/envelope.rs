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

//! Canonical v2 envelope invariants, RFC 8785 digest integrity, and domain payload mappings.

use std::sync::Arc;

use ymir::config::types::{EventNodeIdentity, EventsConfig};
use ymir::oauth::Owner;
use ymir::types::participants::Visibility;
use ymir_events::schemas::dataspace::{
    DataAccessPayload, ObligationExecutionPayload, PolicyDecisionOutcome, PolicyDecisionPayload,
    TransferTransitionPayload,
};
use ymir_events::schemas::identity::{
    GrantAction, GrantDirection, IdentityGrantPayload, ParticipantAction,
    ParticipantRegistryPayload, VerificationAction, VerificationDirection, VerificationPayload,
};
use ymir_events::{
    DeliveryClass, EventCategory, EventContext, EventEnvelope, EventOutcome, EventPayload,
    EventSeverity, EventSource, WebhookDeliveryFormat,
};

fn test_owner() -> Owner {
    Owner::new(
        "user-alice",
        "/admin/upm".parse().unwrap(),
        Visibility::Public,
    )
}

#[test]
fn payload_digest_is_deterministic_under_rfc8785_key_reordering() {
    let first = serde_json::json!({ "b": 2, "a": { "y": true, "x": "hello" } });
    let second = serde_json::json!({ "a": { "x": "hello", "y": true }, "b": 2 });

    let env1 = EventEnvelope::new(
        test_owner(),
        "transfers.started".parse().unwrap(),
        "test-crate",
        1,
        Some("corr-1".to_string()),
        first,
    );
    let env2 = EventEnvelope::new(
        test_owner(),
        "transfers.started".parse().unwrap(),
        "test-crate",
        1,
        Some("corr-1".to_string()),
        second.clone(),
    );

    assert_eq!(env1.spec_version, "2.0");
    assert_eq!(env1.integrity.canonicalization, "rfc8785-sha256");
    assert_eq!(env1.integrity.payload_digest, env2.integrity.payload_digest);
    assert!(env1.integrity.verify_payload(&second));

    let tampered = serde_json::json!({ "a": { "x": "tampered", "y": true }, "b": 2 });
    assert!(!env1.integrity.verify_payload(&tampered));
}

#[test]
fn typed_payloads_populate_canonical_categories_outcomes_and_flow_refs() {
    let ctx = EventContext::for_owner(test_owner())
        .with_peer_id("did:web:peer.example")
        .with_trace_context(Some(
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string(),
        ));

    let grant = IdentityGrantPayload {
        action: GrantAction::Rejected,
        grant_id: "grant-99".to_string(),
        direction: GrantDirection::Received,
        kind: "consumer".to_string(),
        status: "Rejected".to_string(),
        previous_status: Some("Pending".to_string()),
        peer_participant_id: None,
        peer_participant_nick: Some("peer-1".to_string()),
        vc_types: vec!["ParticipantVC".to_string()],
        token_fingerprint: Some(IdentityGrantPayload::fingerprint_token("secret-token-value")),
        expires_at: None,
        reason: Some("signature invalid".to_string()),
    };
    let grant_json = serde_json::to_string(&grant).unwrap();
    assert!(!grant_json.contains("secret-token-value"));
    assert!(grant_json.contains("sha256:"));

    let grant_env = grant
        .into_envelope(&ctx, EventSource::for_crate("heimdall"))
        .unwrap();
    assert_eq!(grant_env.topic.as_str(), "grants:rejected");
    assert_eq!(grant_env.event_type, EventCategory::AuditLifecycle);
    assert_eq!(grant_env.severity, EventSeverity::Warn);
    assert_eq!(grant_env.outcome, Some(EventOutcome::Denied));
    assert_eq!(
        grant_env.correlation.flow_refs.grant_id.as_deref(),
        Some("grant-99")
    );
    assert_eq!(
        grant_env.correlation.correlation_id.as_deref(),
        Some("grant-99")
    );
    assert_eq!(grant_env.subject.as_ref().unwrap().kind, "grant");

    let verification = VerificationPayload {
        action: VerificationAction::Verified,
        grant_id: "grant-99".to_string(),
        verification_id: "ver-1".to_string(),
        direction: VerificationDirection::Incoming,
        holder_did: Some("did:web:holder".to_string()),
        verifier_did: Some("did:web:verifier".to_string()),
        vc_types: vec!["MembershipCredential".to_string()],
        presented_vc_ids: vec!["vc-1".to_string()],
        status: "Verified".to_string(),
        failure_reason: None,
    };
    let ver_env = verification
        .into_envelope(&ctx, EventSource::for_crate("heimdall"))
        .unwrap();
    assert_eq!(ver_env.event_type, EventCategory::Attestation);
    assert_eq!(ver_env.topic.as_str(), "verifications:verified");

    let access = DataAccessPayload {
        allowed: true,
        transfer_id: "tr-1".to_string(),
        agreement_id: Some("ag-1".to_string()),
        dataset_id: Some("ds-1".to_string()),
        channel: "http_proxy".to_string(),
        method: Some("GET".to_string()),
        path: Some("/data".to_string()),
        status_code: Some(200),
        bytes_transferred: Some(1024),
        duration_ms: Some(12),
        peer_id: None,
        denial_reason: None,
    };
    let access_env = access
        .into_envelope(&ctx, EventSource::for_crate("dataplane"))
        .unwrap();
    assert_eq!(access_env.topic.as_str(), "dataplane:data:accessed");
    assert_eq!(access_env.event_type, EventCategory::DataAccess);
    assert_eq!(access_env.delivery_class, DeliveryClass::TelemetryStream);
    assert_eq!(
        access_env.correlation.flow_refs.transfer_id.as_deref(),
        Some("tr-1")
    );
    assert_eq!(
        access_env.correlation.flow_refs.agreement_id.as_deref(),
        Some("ag-1")
    );
    assert_eq!(
        access_env.correlation.flow_refs.dataset_id.as_deref(),
        Some("ds-1")
    );

    let decision = PolicyDecisionPayload {
        decision: PolicyDecisionOutcome::Deny,
        enforcement_point: "dataplane_http".to_string(),
        action: "odrl:use".to_string(),
        agreement_id: Some("ag-1".to_string()),
        transfer_id: Some("tr-1".to_string()),
        dataset_id: Some("ds-1".to_string()),
        policy_id: Some("pol-1".to_string()),
        policy_digest: None,
        peer_id: None,
        obligations_triggered: vec![],
        reason: Some("count limit exceeded".to_string()),
    };
    let decision_env = decision
        .into_envelope(&ctx, EventSource::for_crate("usage-control"))
        .unwrap();
    assert_eq!(decision_env.event_type, EventCategory::UsageControl);
    assert_eq!(decision_env.severity, EventSeverity::Warn);
    assert_eq!(decision_env.outcome, Some(EventOutcome::Denied));

    let obligation = ObligationExecutionPayload {
        obligation_id: "ob-1".to_string(),
        obligation_kind: "delete_after".to_string(),
        enforcement_point: "dataplane_http".to_string(),
        agreement_id: Some("ag-1".to_string()),
        transfer_id: Some("tr-1".to_string()),
        succeeded: false,
        duration_ms: Some(30),
        details: None,
        error_reason: Some("storage error".to_string()),
    };
    let obligation_env = obligation
        .into_envelope(&ctx, EventSource::for_crate("usage-control"))
        .unwrap();
    assert_eq!(obligation_env.severity, EventSeverity::Error);
    assert_eq!(obligation_env.outcome, Some(EventOutcome::Failed));

    let transfer = TransferTransitionPayload {
        transition: "started".to_string(),
        transfer_id: "tr-1".to_string(),
        agreement_id: Some("ag-1".to_string()),
        dataset_id: Some("ds-1".to_string()),
        state: "Started".to_string(),
        previous_state: Some("Requested".to_string()),
        transfer_type: Some("HttpData-PULL".to_string()),
        role: "provider".to_string(),
        counter_party_id: None,
        protocol_event: None,
        error_reason: None,
    };
    let transfer_env = transfer
        .into_envelope(&ctx, EventSource::for_crate("transfer-agent"))
        .unwrap();
    assert_eq!(transfer_env.topic.as_str(), "transfers:started");
    assert_eq!(transfer_env.event_type, EventCategory::AuditLifecycle);

    let participant = ParticipantRegistryPayload {
        action: ParticipantAction::Registered,
        participant_id: "did:web:alice".to_string(),
        participant_nick: Some("Alice Org".to_string()),
        participant_type: Some("LegalPerson".to_string()),
        base_url: None,
        trust_status: Some("trusted".to_string()),
        reason: None,
    };
    let json = serde_json::to_string(&participant).unwrap();
    assert!(!json.contains("private_key"));
    assert!(json.contains("did:web:alice"));
}

#[test]
fn apply_node_identity_fills_unset_fields_without_overwriting_remote_origin() {
    let cfg = EventsConfig {
        dataspace_id: Some("ds-prod".to_string()),
        service_name: Some("monolith".to_string()),
        instance_id: Some("node-1".to_string()),
        ..Default::default()
    };
    let identity = Arc::new(EventNodeIdentity::from_config(&cfg, "monolith"));
    identity.set_participant_id(Some("did:web:authority".to_string()));
    let snap = identity.snapshot();

    let mut local = EventEnvelope::new(
        test_owner(),
        "auth.login".parse().unwrap(),
        "auth",
        1,
        None,
        serde_json::json!({}),
    );
    local.apply_node_identity(&snap);
    assert_eq!(local.source.dataspace_id.as_deref(), Some("ds-prod"));
    assert_eq!(
        local.source.participant_id.as_deref(),
        Some("did:web:authority")
    );
    assert_eq!(local.source.service, "monolith");
    assert_eq!(local.source.instance_id, "node-1");

    let mut remote = local.clone();
    remote.source = EventSource {
        dataspace_id: Some("ds-remote".to_string()),
        participant_id: Some("did:web:worker".to_string()),
        service: "dataplane".to_string(),
        instance_id: "dp-7".to_string(),
        crate_name: "dataplane".to_string(),
    };
    remote.apply_node_identity(&snap);
    assert_eq!(remote.source.dataspace_id.as_deref(), Some("ds-remote"));
    assert_eq!(remote.source.service, "dataplane");
    assert_eq!(remote.source.instance_id, "dp-7");
}

#[test]
fn webhook_delivery_format_parses_aliases() {
    assert_eq!(
        "payload_only".parse::<WebhookDeliveryFormat>().unwrap(),
        WebhookDeliveryFormat::PayloadOnly
    );
    assert_eq!(
        "Envelope".parse::<WebhookDeliveryFormat>().unwrap(),
        WebhookDeliveryFormat::Envelope
    );
}
