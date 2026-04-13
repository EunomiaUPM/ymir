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
use serde_json::Value;

use crate::data::entities::token_requirements::Model;
use crate::types::gnap::grant_request::Access4Req;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manage: Option<Value>,
    pub access: Access4Req,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<Value>,
    pub flags: Option<Vec<String>>,
}

impl AccessToken {
    pub fn new<T: Into<String>>(value: T, model: &Model) -> Self {
        Self {
            value: value.into(),
            label: model.label.clone(),
            manage: None,
            access: Access4Req {
                r#type: model.r#type.clone(),
                actions: Some(model.actions.clone()),
                locations: model.locations.clone(),
                datatypes: model.datatypes.clone(),
                identifier: model.identifier.clone(),
                privileges: model.privileges.clone(),
            },
            expires_in: None,
            key: None,
            flags: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContinueToken {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,
}

impl ContinueToken {
    pub fn new<T: Into<String>>(value: T) -> Self {
        ContinueToken {
            value: value.into(),
            label: None,
            expires_in: None,
            flags: None,
        }
    }
}
