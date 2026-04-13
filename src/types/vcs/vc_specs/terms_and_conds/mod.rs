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

#[derive(Debug, Serialize, Deserialize)]
pub struct TermsAndConditionsCredSub {
    pub id: String,
    #[serde(rename = "gx:url")]
    pub uri: String,
    #[serde(rename = "gx:hash")]
    pub hash: String,
}

impl TermsAndConditionsCredSub {
    pub fn new_gaia(
        id: impl Into<String>,
        uri: impl Into<String>,
        hash: impl Into<String>,
    ) -> TermsAndConditionsCredSub {
        TermsAndConditionsCredSub {
            id: id.into(),
            uri: uri.into(),
            hash: hash.into(),
        }
    }
    pub fn random(id: impl Into<String>) -> TermsAndConditionsCredSub {
        TermsAndConditionsCredSub {
            id: id.into(),
            uri: "https://gaia-x.eu/.well-known/terms-and-conditions.json#cs".to_string(),
            hash: "067dcac5efd18c1927deb1ffed3feab6d0ad044c0a9a263e6d5d8bdc43224515".to_string(),
        }
    }
}
