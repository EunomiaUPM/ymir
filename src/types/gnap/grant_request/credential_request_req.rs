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

use crate::types::gnap::grant_request::{Access4Req, InteractActions};
use crate::types::vcs::VcType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CredentialRequest4GR {
    pub access: Access4Req,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>, // REQUIRED if used as part of a request for multiple access tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
}

impl CredentialRequest4GR {
    pub fn new(vc_type: &VcType) -> Self {
        CredentialRequest4GR {
            access: Access4Req {
                r#type: "vc-exchange".to_string(),
                actions: Some(vec![InteractActions::RequestVc.to_string()]),
                locations: None,
                datatypes: Some(vec![vc_type.to_conf()]),
                identifier: None,
                privileges: None,
            },
            label: None,
            flags: None,
        }
    }
}
