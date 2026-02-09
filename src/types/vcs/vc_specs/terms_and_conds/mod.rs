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

use crate::types::vcs::vc_specs::BaseCredentialSubject;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TermsAndConditionsCredSub {
    #[serde(flatten)]
    pub base: BaseCredentialSubject,
    #[serde(rename = "gx:url")]
    pub url: String,
    #[serde(rename = "gx:hash")]
    pub hash: String,
}

impl TermsAndConditionsCredSub {
    pub fn new_gaia(kid: &str) -> TermsAndConditionsCredSub {
        Self {
            base: BaseCredentialSubject {
                id: kid.to_string(),
                r#type: "gx:TermsAndConditions".to_string(),
            },
            url: "test_url".to_string(),
            hash: "test_hash".to_string(),
        }
    }
}
