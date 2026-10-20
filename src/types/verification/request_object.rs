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

//! The signed Request Object of an OpenID4VP 1.0 request passed by reference (`request_uri`,
//! JAR / RFC 9101): what the Verifier serves and the wallet verifies.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::crypto::{HasProofPurpose, ProofPurpose};
use crate::types::jwt::{HasJwtType, JwtType};
use crate::types::verification::DcqlQuery;

/// Media type the Request Object is served with (RFC 9101 §5.2.3).
pub const REQUEST_OBJECT_MEDIA_TYPE: &str = "application/oauth-authz-req+jwt";
/// `aud` of a Request Object under Static Discovery (OpenID4VP 1.0 §5.8).
pub const SELF_ISSUED_AUDIENCE: &str = "https://self-issued.me/v2";

/// What a Request Object carries besides its registered claims (`iss` = the Client Identifier,
/// `aud`, `nonce`, `iat`, `exp`): the Authorization Request parameters.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestObjectPayload {
    /// Must equal the `client_id` of the URI that pointed here.
    pub client_id: String,
    pub response_type: String,
    pub response_mode: String,
    pub response_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub dcql_query: DcqlQuery,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_metadata: Option<Value>,
    /// The `wallet_nonce` the wallet posted to the `request_uri`, if any (§5.10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_nonce: Option<String>,
}

/// Signing a request authenticates the Verifier, as a presentation does the holder.
impl HasProofPurpose for RequestObjectPayload {
    const PURPOSE: ProofPurpose = ProofPurpose::Authentication;
}

/// Wallets must refuse any other `typ` (OpenID4VP 1.0 §5).
impl HasJwtType for RequestObjectPayload {
    const TYP: JwtType = JwtType::RequestObject;
}

/// What a wallet may post to a `request_uri` with `request_uri_method=post` (§5.10), as a form.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RequestUriPost {
    /// The wallet's metadata, a JSON object as a string.
    #[serde(default)]
    pub wallet_metadata: Option<String>,
    /// To be echoed in the Request Object, against replay.
    #[serde(default)]
    pub wallet_nonce: Option<String>,
}
