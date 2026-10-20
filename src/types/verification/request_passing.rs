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

/// How the Authorization Request reaches the wallet (OpenID4VP 1.0 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum RequestPassing {
    /// The URI carries only `client_id` and a `request_uri`; the wallet fetches from there a
    /// Request Object signed with the Verifier's DID (`decentralized_identifier:` prefix).
    #[default]
    ByReference,
    /// Every parameter travels in the URI, unsigned (`redirect_uri:` prefix): the wallet cannot
    /// tell who the Verifier is.
    ByValue,
}