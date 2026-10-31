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

/// Locator for a DID stored in the wallet.
///
/// Allows wallet APIs to address an entry either by the internal id assigned
/// at registration (`Id`) or by the DID string itself (`Did`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DidSearch {
    Id(String),
    Did(String),
}

impl DidSearch {
    pub fn as_str(&self) -> &str {
        match self {
            DidSearch::Id(s) | DidSearch::Did(s) => s.as_str(),
        }
    }
}
