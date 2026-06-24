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
use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::impl_serde_via_str;
use crate::types::keys::Alg;
use crate::types::vcs::{VcFormat, VcType};

// ════════════════════════════════════════════════════════════════════════════════
//   CredentialConfiguration
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialConfiguration {
    /// OAuth 2.0 scope value used to request this credential. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Cryptographic binding methods supported (e.g. `did:jwk`, `jwk`, `cose_key`).
    /// OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cryptographic_binding_methods_supported: Option<Vec<CryptoBindingMethod>>,

    /// Signing algorithms (JWA) used by the issuer when signing this credential.
    /// OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_signing_alg_values_supported: Option<Vec<Alg>>,

    /// Supported proof types and their parameters, keyed by proof type identifier.
    /// OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_types_supported: Option<HashMap<ProofType, ProofTypeMetadata>>,

    /// Display + claims metadata for this credential. Format-specific mechanisms
    /// (e.g. SD-JWT VC display metadata) take precedence; this serves as
    /// fallback default. OIDC4VCI 1.0 §11.2.3. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_metadata: Option<CredentialMetadata>,

    /// Format + the format-specific fields it requires. Wire JSON: `format` plus
    /// the variant fields appear flattened at this struct's level.
    #[serde(flatten)]
    pub format_data: FormatSpecific,
}

impl CredentialConfiguration {
    /// Returns the credential format as a `VcFormat`.
    pub fn format(&self) -> VcFormat {
        self.format_data.format()
    }

    /// Build a `jwt_vc_json` credential configuration with sensible defaults.
    ///
    /// Defaults applied:
    /// - `cryptographic_binding_methods_supported`: `[did:jwk, did:web]`
    /// - `credential_signing_alg_values_supported`: `Alg::supported()`
    /// - `proof_types_supported`: `{ "jwt": { algs: Alg::supported() } }`
    ///
    /// `scope` and `credential_metadata` are left as `None`. Override any field
    /// via direct field access after construction if you need to customize.
    pub fn jwt_vc_json(vc_type: &VcType) -> Self {
        let mut proof_types = HashMap::new();
        proof_types.insert(
            ProofType::Jwt,
            ProofTypeMetadata {
                proof_signing_alg_values_supported: Alg::supported(),
                key_attestations_required: None,
            },
        );

        Self {
            scope: None,
            cryptographic_binding_methods_supported: Some(vec![
                CryptoBindingMethod::DidJwk,
                CryptoBindingMethod::DidWeb,
            ]),
            credential_signing_alg_values_supported: Some(Alg::supported()),
            proof_types_supported: Some(proof_types),
            credential_metadata: None,
            format_data: FormatSpecific::JwtVcJson {
                credential_definition: CredentialDefinition {
                    r#type: vec!["VerifiableCredential".to_string(), vc_type.to_string()],
                    context: None,
                },
            },
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
//   CredentialMetadata
// ════════════════════════════════════════════════════════════════════════════════

/// Wrapper for credential display and claims metadata (OIDC4VCI 1.0 §11.2.3).
///
/// Format-specific mechanisms (SD-JWT VC display metadata, etc.) override these
/// values when present. This is the format-agnostic fallback used by Wallets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    /// Display information for rendering to end-users. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Vec<CredentialDisplay>>,

    /// Description of the claims contained in this credential (Appendix B.2).
    /// OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claims: Option<Vec<ClaimMetadata>>,
}

// ════════════════════════════════════════════════════════════════════════════════
//   FormatSpecific
// ════════════════════════════════════════════════════════════════════════════════

/// Format-discriminated payload for a credential configuration.
///
/// Serializes with the `format` field as the discriminator, and the variant's
/// fields flattened alongside it. Combined with `#[serde(flatten)]` in
/// `CredentialConfiguration`, the result matches the OIDC4VCI 1.0 wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format")]
pub enum FormatSpecific {
    #[serde(rename = "jwt_vc_json")]
    JwtVcJson {
        credential_definition: CredentialDefinition,
    },

    #[serde(rename = "jwt_vc_json-ld")]
    JwtVcJsonLd {
        credential_definition: CredentialDefinition,
    },

    #[serde(rename = "ldp_vc")]
    LdpVc {
        credential_definition: CredentialDefinition,
    },

    #[serde(rename = "vc+sd-jwt")]
    SdJwtVc { vct: String },

    #[serde(rename = "mso_mdoc")]
    MsoMdoc { doctype: String },
}

impl FormatSpecific {
    /// Returns the format as a `VcFormat`.
    pub fn format(&self) -> VcFormat {
        match self {
            Self::JwtVcJson { .. } => VcFormat::JwtVcJson,
            Self::JwtVcJsonLd { .. } => VcFormat::JwtVcJsonLd,
            Self::LdpVc { .. } => VcFormat::LdpVc,
            Self::SdJwtVc { .. } => VcFormat::SdJwtVc,
            Self::MsoMdoc { .. } => VcFormat::MsoMdoc,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
//   CryptoBindingMethod
// ════════════════════════════════════════════════════════════════════════════════

/// Cryptographic binding methods accepted by the issuer for binding the holder's
/// public key to the credential. OIDC4VCI 1.0 §12.2.4.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CryptoBindingMethod {
    /// JWK (RFC 7517).
    Jwk,
    /// COSE Key (RFC 8152).
    CoseKey,
    /// Any DID method.
    Did,
    /// `did:jwk`.
    DidJwk,
    /// `did:key`.
    DidKey,
    /// `did:web`.
    DidWeb,
    /// `did:ebsi`.
    DidEbsi,
    /// `did:ion`.
    DidIon,
    /// Unknown / future binding method.
    Other(String),
}

impl Display for CryptoBindingMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Jwk => "jwk",
            Self::CoseKey => "cose_key",
            Self::Did => "did",
            Self::DidJwk => "did:jwk",
            Self::DidKey => "did:key",
            Self::DidWeb => "did:web",
            Self::DidEbsi => "did:ebsi",
            Self::DidIon => "did:ion",
            Self::Other(other) => other.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for CryptoBindingMethod {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "jwk" => Self::Jwk,
            "cose_key" => Self::CoseKey,
            "did" => Self::Did,
            "did:jwk" => Self::DidJwk,
            "did:key" => Self::DidKey,
            "did:web" => Self::DidWeb,
            "did:ebsi" => Self::DidEbsi,
            "did:ion" => Self::DidIon,
            other => Self::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(CryptoBindingMethod);

// ════════════════════════════════════════════════════════════════════════════════
//   ProofType
// ════════════════════════════════════════════════════════════════════════════════

/// Proof types the issuer accepts in the holder's credential request.
/// Used as keys of `proof_types_supported`. OIDC4VCI 1.0 §8.2.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProofType {
    /// JWT proof per JOSE (RFC 7515).
    Jwt,
    /// Linked Data Proof Verifiable Presentation.
    LdpVp,
    /// Data Integrity Verifiable Presentation.
    DiVp,
    /// Key attestation as proof.
    Attestation,
    /// Unknown / future-proof type.
    Other(String),
}

impl Display for ProofType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Jwt => "jwt",
            Self::LdpVp => "ldp_vp",
            Self::DiVp => "di_vp",
            Self::Attestation => "attestation",
            Self::Other(other) => other.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for ProofType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "jwt" => Self::Jwt,
            "ldp_vp" => Self::LdpVp,
            "di_vp" => Self::DiVp,
            "attestation" => Self::Attestation,
            other => Self::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(ProofType);

// ════════════════════════════════════════════════════════════════════════════════
//   Subestructuras
// ════════════════════════════════════════════════════════════════════════════════

/// Metadata for one supported proof type. Used as value in `proof_types_supported`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofTypeMetadata {
    /// Algorithms (JWA) accepted in the holder's proof. REQUIRED.
    pub proof_signing_alg_values_supported: Vec<Alg>,

    /// Additional key attestations the issuer requires. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_attestations_required: Option<KeyAttestationsRequired>,
}

/// Key attestations the issuer requires in the holder's proof.
/// Values are kept as `String` because they belong to evolving attestation frameworks
/// (ISO 18045 levels, etc.) and may differ across deployments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAttestationsRequired {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_storage: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_authentication: Option<Vec<String>>,
}

/// Credential Definition for `jwt_vc_json`, `jwt_vc_json-ld`, `ldp_vc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialDefinition {
    /// The credential `type` array (W3C VC).
    #[serde(rename = "type")]
    pub r#type: Vec<String>,

    /// JSON-LD `@context`. REQUIRED for `jwt_vc_json-ld` and `ldp_vc`,
    /// MUST NOT be present for `jwt_vc_json`.
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<String>>,
}

// ─── Display ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialDisplay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<DisplayLogo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_image: Option<DisplayImage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayLogo {
    pub uri: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayImage {
    pub uri: String,
}

// ─── Claims ────────────────────────────────────────────────────────────────────

/// One claim definition entry. See OIDC4VCI 1.0 Appendix B.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimMetadata {
    /// Path to the claim in the credential's structured payload.
    /// Each element is a key (string), an array index (number), or null (all).
    pub path: Vec<Option<ClaimPathSegment>>,

    /// Whether this claim is mandatory. OPTIONAL, default false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mandatory: Option<bool>,

    /// Display information for this claim per locale. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Vec<ClaimDisplay>>,
}

/// A path segment inside a `ClaimMetadata.path`.
/// `Vec<Option<ClaimPathSegment>>` uses `None` to represent JSON `null`
/// (meaning "all elements of this array").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClaimPathSegment {
    /// Object key.
    Key(String),
    /// Array index.
    Index(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimDisplay {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
