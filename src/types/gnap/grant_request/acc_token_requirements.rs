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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use serde::{Deserialize, Serialize};
use crate::types::gnap::gr_use::GRUse;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessTokenRequirements4GR {
    pub access: Access4AT,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>, // REQUIRED if used as part of a request for multiple access tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>, /* A set of flags that indicate desired attributes or behavior to be attached
                                     * to the access token by the AS */
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Access4AT {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<String>>, // Actions4Access4AT COMPLETAR
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datatypes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privileges: Option<Vec<String>>,
}

pub enum TokenReqTypeGR {
    Key,
    Bearer,
}

impl AccessTokenRequirements4GR {
    pub fn new(option: GRUse, token: Option<TokenReqTypeGR>) -> Self {
        let mut data = AccessTokenRequirements4GR::default();

        data.access.r#type = match option {
            GRUse::Talk => "api-access",
            GRUse::VcReq => "vc-exchange",
        }
        .to_string();

        if let Some(TokenReqTypeGR::Bearer) = token {
            data.flags = Some(vec!["Bearer".to_string()]);
        }

        data
    }
}

impl Default for AccessTokenRequirements4GR {
    fn default() -> Self {
        Self {
            access: Access4AT {
                r#type: "".to_string(),
                actions: Some(vec!["talk".to_string()]),
                locations: None,
                datatypes: None,
                identifier: None,
                privileges: None,
            },
            label: None,
            flags: None,
        }
    }
}
