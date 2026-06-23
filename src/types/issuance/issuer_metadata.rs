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

use std::collections::HashMap;

use super::{CredentialConfiguration, DisplayLogo};
use crate::types::vcs::{VcFormat, VcType, VcTypeConfig};
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════════════════════
//   IssuerMetadata
// ════════════════════════════════════════════════════════════════════════════════

/// Credential Issuer Metadata document (OIDC4VCI 1.0 §12.2.4).
///
/// Published at `/.well-known/openid-credential-issuer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerMetadata {
    /// URL identifying the Credential Issuer. REQUIRED.
    pub credential_issuer: String,

    /// URL of the Credential Endpoint. REQUIRED.
    pub credential_endpoint: String,

    /// Map of credential_configuration_id → configuration. REQUIRED.
    pub credential_configurations_supported: HashMap<VcTypeConfig, CredentialConfiguration>,

    /// Authorization Servers protecting this issuer's endpoints. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_servers: Option<Vec<String>>,

    /// URL of the Nonce Endpoint (returns fresh `c_nonce`). OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce_endpoint: Option<String>,

    /// URL of the Deferred Credential Endpoint. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deferred_credential_endpoint: Option<String>,

    /// URL of the Notification Endpoint. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_endpoint: Option<String>,

    /// JWE encryption settings for the Credential Response. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_response_encryption: Option<CredentialResponseEncryption>,

    /// JWE encryption settings for the Credential Request (§8.2). OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_request_encryption: Option<CredentialRequestEncryption>,

    /// Batch issuance capability. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_credential_issuance: Option<BatchCredentialIssuance>,

    /// JWT carrying the same metadata signed by the issuer. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_metadata: Option<String>,

    /// Issuer branding for wallet UI. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Vec<IssuerDisplay>>,
}

impl IssuerMetadata {
    pub fn new(issuer: &str, api_path: &str, vc_types: &[VcType]) -> Self {
        let mut supported: HashMap<VcTypeConfig, CredentialConfiguration> = HashMap::new();
        let formats = VcFormat::supported();

        for vc_type in vc_types {
            for format in formats {
                let config = VcTypeConfig::new(vc_type.clone(), format.clone());
                let cred_config = match format {
                    VcFormat::JwtVcJson => CredentialConfiguration::jwt_vc_json(config.vc_type()),
                    _ => continue,
                };
                supported.insert(config, cred_config);
            }
        }

        Self {
            credential_issuer: issuer.to_string(),
            credential_endpoint: format!("{}{}/credential", issuer, api_path),
            credential_configurations_supported: supported,
            authorization_servers: Some(vec![issuer.to_string()]),
            nonce_endpoint: None,
            deferred_credential_endpoint: None,
            notification_endpoint: None,
            credential_response_encryption: None,
            credential_request_encryption: None,
            batch_credential_issuance: None,
            signed_metadata: None,
            display: None,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
//   CredentialResponseEncryption
// ════════════════════════════════════════════════════════════════════════════════

/// JWE settings for encrypting the Credential Response.
///
/// `alg_values_supported` and `enc_values_supported` use JOSE identifiers (e.g.
/// `"RSA-OAEP"`, `"ECDH-ES"` for `alg`; `"A256GCM"`, `"A128CBC-HS256"` for `enc`).
/// Kept as `Vec<String>` because the JWE algorithm set is wide and we don't have a
/// typed enum for them yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialResponseEncryption {
    /// JWE `alg` (key wrapping) values the issuer supports. REQUIRED.
    pub alg_values_supported: Vec<String>,

    /// JWE `enc` (content encryption) values the issuer supports. REQUIRED.
    pub enc_values_supported: Vec<String>,

    /// If true, the wallet MUST decrypt the credential response. REQUIRED.
    pub encryption_required: bool,
}

// ════════════════════════════════════════════════════════════════════════════════
//   CredentialRequestEncryption
// ════════════════════════════════════════════════════════════════════════════════

/// JWE settings for the wallet to encrypt the Credential Request (§8.2).
///
/// Same structure as `CredentialResponseEncryption` but applies to the request
/// direction. Both fields use JOSE identifiers for `alg` and `enc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRequestEncryption {
    /// JWE `alg` (key wrapping) values the issuer accepts. REQUIRED.
    pub alg_values_supported: Vec<String>,

    /// JWE `enc` (content encryption) values the issuer accepts. REQUIRED.
    pub enc_values_supported: Vec<String>,

    /// If true, the wallet MUST encrypt its credential request. REQUIRED.
    pub encryption_required: bool,
}

// ════════════════════════════════════════════════════════════════════════════════
//   BatchCredentialIssuance
// ════════════════════════════════════════════════════════════════════════════════

/// Batch issuance configuration. Present iff the issuer supports batch requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCredentialIssuance {
    /// Maximum number of credentials per batch request. REQUIRED, MUST be ≥ 2.
    pub batch_size: u32,
}

// ════════════════════════════════════════════════════════════════════════════════
//   IssuerDisplay
// ════════════════════════════════════════════════════════════════════════════════

/// Issuer display info per locale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerDisplay {
    /// Display name of the issuer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// BCP47 language tag (e.g. `"en"`, `"es"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// Logo of the issuer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<DisplayLogo>,
}
