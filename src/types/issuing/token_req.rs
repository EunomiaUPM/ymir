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

use serde::Deserialize;

use super::OidcGrantType;

/// Token Request received by the AS Token Endpoint (OIDC4VCI 1.0 §6.1).
///
/// Sent by the wallet as `application/x-www-form-urlencoded`.
/// This struct covers the Pre-Authorized Code flow; the auth_code-specific
/// fields (`code`, `redirect_uri`, `code_verifier`) are not modeled here.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    /// The grant type the wallet is using. REQUIRED.
    pub grant_type: OidcGrantType,

    /// The pre-authorized code received in the Credential Offer. REQUIRED for
    /// the pre-authorized code grant. JSON name MUST be `pre-authorized_code`
    /// (with hyphen).
    #[serde(rename = "pre-authorized_code")]
    pub pre_authorized_code: String,

    /// Transaction Code value, when the offer required one. OPTIONAL.
    #[serde(default)]
    pub tx_code: Option<String>,

    /// Client identifier, when client authentication is used. OPTIONAL for the
    /// pre-authorized code grant.
    #[serde(default)]
    pub client_id: Option<String>,
}
