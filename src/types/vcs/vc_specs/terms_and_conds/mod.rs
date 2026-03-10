/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
    #[serde(rename = "gx:url")]
    pub uri: String,
    #[serde(rename = "gx:hash")]
    pub hash: String
}

impl TermsAndConditionsCredSub {
    pub fn new_gaia(uri: impl Into<String>, hash: impl Into<String>) -> TermsAndConditionsCredSub {
        TermsAndConditionsCredSub { uri: uri.into(), hash: hash.into() }
    }
    pub fn random() -> TermsAndConditionsCredSub {
        TermsAndConditionsCredSub { uri: "uri_to_stuff".to_string(), hash: "kk".to_string() }
    }
}
