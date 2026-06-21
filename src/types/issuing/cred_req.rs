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

use serde::{Deserialize, Serialize};

use crate::types::vcs::VcTypeConfig;

// ════════════════════════════════════════════════════════════════════════════════
//   CredentialRequest
// ════════════════════════════════════════════════════════════════════════════════

/// Credential Request received at the Credential Endpoint (OIDC4VCI 1.0 §8.1).
///
/// Either `credential_configuration_id` OR `credential_identifier` MUST be
/// present (mutually exclusive). `proof` is required when the issuer mandates
/// proof of possession (our case for `jwt_vc_json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRequest {
    /// References a key in `credential_configurations_supported` of the Issuer
    /// Metadata. REQUIRED when no `credential_identifier` is present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_configuration_id: Option<VcTypeConfig>,

    /// Refers to a specific Credential previously authorised via
    /// `authorization_details`. REQUIRED when the AS returned identifiers in
    /// the token response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_identifier: Option<String>,

    /// Single proof of possession of the holder's key. REQUIRED when the
    /// issuer's configuration requires proofs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<CredReqProof>,

    /// Multiple proofs (for batch issuance). Alternative to `proof`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proofs: Option<CredReqProofs>,

    /// JWE parameters for encrypting the credential response. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_response_encryption: Option<serde_json::Value>,
}

// ════════════════════════════════════════════════════════════════════════════════
//   CredReqProof
// ════════════════════════════════════════════════════════════════════════════════

/// Proof of possession of the holder's key, discriminated by `proof_type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "proof_type", rename_all = "snake_case")]
pub enum CredReqProof {
    /// JWT proof (OIDC4VCI 1.0 §8.2.1.1). Header `typ` MUST be
    /// `openid4vci-proof+jwt`. Payload MUST contain `aud`, `iat`, and `nonce`
    /// (echoing the `c_nonce` from the token response).
    Jwt { jwt: String },

    /// W3C Linked-Data Proof carried inside a Verifiable Presentation.
    LdpVp { ldp_vp: serde_json::Value },

    /// Attestation-based proof.
    Attestation { attestation: String },
}

// ════════════════════════════════════════════════════════════════════════════════
//   CredReqProofs
// ════════════════════════════════════════════════════════════════════════════════

/// Batch of proofs of the same type for batch issuance.
///
/// Per spec, only one variant is populated per request, matching the chosen
/// proof_type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CredReqProofs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldp_vp: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<Vec<String>>,
}
