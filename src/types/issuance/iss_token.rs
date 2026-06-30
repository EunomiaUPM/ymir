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

use crate::utils::create_opaque_token;

/// Token Response returned by the AS Token Endpoint (OIDC4VCI 1.0 §6.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuingToken {
    /// Access token the wallet uses as Bearer at the Credential Endpoint.
    /// REQUIRED.
    pub access_token: String,

    /// Token type. Typically `"Bearer"`; `"DPoP"` if DPoP is in use.
    /// REQUIRED.
    pub token_type: String,

    /// Lifetime of `access_token` in seconds. RECOMMENDED.
    pub expires_in: u32,

    /// Nonce the wallet MUST include in its JWT proof at the Credential Endpoint.
    /// OPTIONAL — required only if the issuer mandates proof of possession.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_nonce: Option<String>,

    /// Lifetime of `c_nonce` in seconds. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_nonce_expires_in: Option<u32>,

    /// Refresh token if the issuer supports refresh flows. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// Scopes the access token is restricted to. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Authorization details per RFC 9396 with OIDC4VCI extensions. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_details: Option<serde_json::Value>,
}

impl IssuingToken {
    /// Build a token response with an opaque access token and the given
    /// lifetime. If `nonce` is `Some`, it's written to `c_nonce` and
    /// `c_nonce_expires_in` is set to `DEFAULT_NONCE_EXPIRES_IN`. If `None`,
    /// both nonce fields stay `None`.
    ///
    /// `refresh_token`, `scope`, and `authorization_details` are left as
    /// `None`; set them via field access if needed.
    pub fn new(token: impl Into<String>, nonce: Option<String>, expires_in: u32) -> Self {
        let c_nonce_expires_in = nonce.as_ref().map(|_| 3600);

        Self {
            access_token: token.into(),
            token_type: "Bearer".to_string(),
            expires_in,
            c_nonce: nonce,
            c_nonce_expires_in,
            refresh_token: None,
            scope: None,
            authorization_details: None,
        }
    }
}

impl Default for IssuingToken {
    fn default() -> Self {
        Self {
            access_token: create_opaque_token(),
            token_type: "Bearer".to_string(),
            expires_in: 600,
            c_nonce: None,
            c_nonce_expires_in: None,
            refresh_token: None,
            scope: None,
            authorization_details: None,
        }
    }
}
