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

use serde::Serialize;
use crate::types::vcs::VPDef;
use crate::types::verification::DcqlQuery;

/// What a verifier asks for: a Presentation Definition (draft 20) or a DCQL query (1.0).
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Requested {
    Definition(VPDef),
    Dcql(DcqlQuery),
}

/// An OpenID4VP Authorization Request, whatever its version.
#[derive(Debug, Serialize)]
pub struct PresentationRequest {
    /// Full Client Identifier: the `aud` of every presentation.
    pub client_id: String,
    /// Where the answer is posted, as the verifier sent it.
    pub response_uri: String,
    pub nonce: String,
    /// Echoed back with the answer when present.
    pub state: Option<String>,
    pub requested: Requested,
}
