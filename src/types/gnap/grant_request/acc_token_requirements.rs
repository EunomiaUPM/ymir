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

use super::InteractActions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessTokenRequirements4GR {
    pub access: Access4Req,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>, // REQUIRED if used as part of a request for multiple access tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>> /* A set of flags that indicate desired attributes or behavior to be attached
                                    * to the access token by the AS */
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Access4Req {
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
    pub privileges: Option<Vec<String>>
}

pub enum TokenReqTypeGR {
    Key,
    Bearer
}

impl AccessTokenRequirements4GR {
    pub fn new(actions: Option<&[InteractActions]>) -> Self {
        let actions_vec: Vec<String> = actions
            .map_or(vec![InteractActions::Talk], |a| a.to_vec())
            .into_iter()
            .map(|a| a.to_string())
            .collect();
        AccessTokenRequirements4GR {
            access: Access4Req {
                r#type: "api-access".to_string(),
                actions: Some(actions_vec),
                locations: None,
                datatypes: None,
                identifier: None,
                privileges: None
            },
            label: None,
            flags: None
        }
    }
}
