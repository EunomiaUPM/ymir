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

// ════════════════════════════════════════════════════════════════════════════════
//   GiveVC
// ════════════════════════════════════════════════════════════════════════════════

/// Credential Response returned by the Credential Endpoint (OIDC4VCI 1.0 §8.3).
///
/// Either `credentials` (synchronous) or `transaction_id` (deferred) is
/// present. Both being `None` is a protocol error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiveVC {
    /// Issued credentials. REQUIRED for synchronous issuance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Vec<IssuedCredentialItem>>,

    /// Identifier the wallet uses to poll the Deferred Credential Endpoint.
    /// REQUIRED for deferred issuance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,

    /// Identifier the wallet uses to send events to the Notification Endpoint.
    /// OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_id: Option<String>,
}

impl GiveVC {
    /// Build a synchronous response wrapping one or more issued credentials.
    pub fn synchronous(credentials: Vec<IssuedCredential>) -> Self {
        Self {
            credentials: Some(
                credentials
                    .into_iter()
                    .map(|c| IssuedCredentialItem { credential: c })
                    .collect(),
            ),
            transaction_id: None,
            notification_id: None,
        }
    }

    /// Build a deferred response containing only a transaction id.
    pub fn deferred(transaction_id: impl Into<String>) -> Self {
        Self {
            credentials: None,
            transaction_id: Some(transaction_id.into()),
            notification_id: None,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
//   IssuedCredentialItem
// ════════════════════════════════════════════════════════════════════════════════

/// One credential inside the response's `credentials` array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedCredentialItem {
    /// The issued credential, either as a compact JWT string or as a JSON-LD
    /// document.
    pub credential: IssuedCredential,
}

// ════════════════════════════════════════════════════════════════════════════════
//   IssuedCredential
// ════════════════════════════════════════════════════════════════════════════════

/// Serialized form of an issued credential.
///
/// `#[serde(untagged)]` so the wire format matches the spec: a bare string for
/// JWT-based formats (`jwt_vc_json`, `sd_jwt_vc`, `mso_mdoc`, …), a bare JSON
/// object for `ldp_vc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IssuedCredential {
    /// JWT-based credential (compact serialization).
    Jwt(String),

    /// JSON-LD credential (full document with embedded proof).
    JsonLd(serde_json::Value),
}

impl IssuedCredential {
    /// Convenience constructor for the JWT case (the common one in our setup).
    pub fn jwt(signed: impl Into<String>) -> Self {
        Self::Jwt(signed.into())
    }
}
