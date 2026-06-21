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
//   VcCredOffer
// ════════════════════════════════════════════════════════════════════════════════

/// Credential Offer Object (OIDC4VCI 1.0 §4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcCredOffer {
    /// URL of the Credential Issuer. REQUIRED.
    pub credential_issuer: String,

    /// Non-empty array of credential_configuration_ids referencing
    /// `credential_configurations_supported` in the Issuer Metadata. REQUIRED.
    pub credential_configuration_ids: Vec<String>,

    /// Grants the wallet can use to obtain a token. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grants: Option<CredOfferGrants>,
}

impl VcCredOffer {
    /// Build a Credential Offer for the Pre-Authorized Code flow.
    ///
    /// The wallet exchanges the `pre_authorized_code` at the token endpoint to
    /// obtain an access token, then calls the credential endpoint.
    pub fn pre_authorized(
        issuer: impl Into<String>,
        pre_authorized_code: impl Into<String>,
        configurations: &[VcTypeConfig],
    ) -> Self {
        Self {
            credential_issuer: issuer.into(),
            credential_configuration_ids: configurations
                .iter()
                .map(ToString::to_string)
                .collect(),
            grants: Some(CredOfferGrants {
                authorization_code: None,
                pre_authorized_code: Some(PreAuthorizedCodeGrant {
                    pre_authorized_code: pre_authorized_code.into(),
                    tx_code: None,
                    authorization_server: None,
                }),
            }),
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
//   CredOfferGrants
// ════════════════════════════════════════════════════════════════════════════════

/// Map of supported grant types in the Credential Offer.
///
/// At least one variant SHOULD be present per the spec. Both may coexist to let
/// the wallet choose.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CredOfferGrants {
    /// Authorization Code grant. OPTIONAL.
    #[serde(rename = "authorization_code", skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<AuthorizationCodeGrant>,

    /// Pre-Authorized Code grant. OPTIONAL.
    #[serde(
        rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code",
        skip_serializing_if = "Option::is_none"
    )]
    pub pre_authorized_code: Option<PreAuthorizedCodeGrant>,
}

// ════════════════════════════════════════════════════════════════════════════════
//   AuthorizationCodeGrant
// ════════════════════════════════════════════════════════════════════════════════

/// Authorization Code grant parameters in a Credential Offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCodeGrant {
    /// Opaque value created by the issuer to bind the subsequent Authorization
    /// Request. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer_state: Option<String>,

    /// Authorization Server identifier when the issuer lists more than one.
    /// OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_server: Option<String>,
}

// ════════════════════════════════════════════════════════════════════════════════
//   PreAuthorizedCodeGrant
// ════════════════════════════════════════════════════════════════════════════════

/// Pre-Authorized Code grant parameters in a Credential Offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreAuthorizedCodeGrant {
    /// Short-lived single-use code that the wallet exchanges at the token
    /// endpoint. REQUIRED. JSON name MUST be `pre-authorized_code` (with hyphen).
    #[serde(rename = "pre-authorized_code")]
    pub pre_authorized_code: String,

    /// Transaction Code requirements. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_code: Option<TxCodeConfig>,

    /// Authorization Server identifier when the issuer lists more than one.
    /// OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_server: Option<String>,
}

// ════════════════════════════════════════════════════════════════════════════════
//   TxCodeConfig
// ════════════════════════════════════════════════════════════════════════════════

/// Transaction Code requirements presented to the holder. All fields OPTIONAL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxCodeConfig {
    /// Character set accepted in the Transaction Code. Default is `numeric`.
    /// OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_mode: Option<TxCodeInputMode>,

    /// Expected length of the Transaction Code. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,

    /// Guidance for the holder on how to obtain the Transaction Code.
    /// Max 300 characters. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Character set for a Transaction Code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxCodeInputMode {
    /// Only digits.
    Numeric,
    /// Any characters.
    Text,
}

// ════════════════════════════════════════════════════════════════════════════════
//   VcTransmissionOffer (internal helper, not part of the spec wire format)
// ════════════════════════════════════════════════════════════════════════════════

/// How the issuer hands the credential offer to the wallet.
///
/// Not part of the OIDC4VCI wire format — used by the issuer service to decide
/// how to build the deep-link URI.
pub enum VcTransmissionOffer {
    /// Pass the offer by reference: the wallet fetches it from
    /// `credential_offer_uri` using the given id.
    ByReference(String),

    /// Pass the offer by value: the full `VcCredOffer` JSON is embedded in
    /// `credential_offer`. The issuance::Model carries the data needed to
    /// build the offer.
    ByValue(VcCredOffer),
}
