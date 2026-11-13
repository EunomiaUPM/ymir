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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde::{Deserialize, Serialize};

use crate::types::vcs::VcType;

/// Asks a Gaia-X compliance service for a credential by plain POST: a presentation of the
/// newest usable credential of each type goes to `url`, and the credential it answers is stored.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComplianceRequest {
    /// The compliance endpoint, such as Loire's `.../credential-offers/standard-compliance`.
    pub url: String,
    /// The `vcid` of the credential to issue; a fresh `urn:uuid:` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vcid: Option<String>,
    pub types: Vec<VcType>,
}
