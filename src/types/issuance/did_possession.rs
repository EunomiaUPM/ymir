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
//   DidPossession
// ════════════════════════════════════════════════════════════════════════════════

/// Payload claims of a JWT proof of possession (OIDC4VCI 1.0 §8.2.1.1).
///
/// The JWT header MUST carry `alg`, `typ = "openid4vci-proof+jwt"`, and one of
/// `kid` / `jwk` / `x5c` identifying the holder's key. Those are handled by the
/// JWT library used to sign/verify, not in this struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidPossession {
    /// Wallet's client identifier. OPTIONAL — omitted in anonymous flows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// Credential Issuer URL. REQUIRED — anti-replay across issuers.
    pub aud: String,

    /// Issued-at time as Unix timestamp (seconds). REQUIRED.
    pub iat: i64,

    /// Echo of the `c_nonce` issued by the AS. REQUIRED when `c_nonce` was
    /// provided in the token response.
    pub nonce: String,
}
