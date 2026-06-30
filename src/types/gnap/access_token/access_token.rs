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

use crate::data::entities::shared::resource_req;
use crate::types::gnap::grant_request::access::{AccessTokenFlag, ResourceAccess};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manage: Option<Value>,
    pub access: ResourceAccess,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<AccessTokenFlag>>,
}

impl AccessToken {
    pub fn new(token: impl Into<String>, model: resource_req::Model) -> Self {
        Self {
            value: token.into(),
            label: model.label,
            manage: None,
            access: ResourceAccess {
                r#type: model.r#type,
                actions: Some(model.actions),
                locations: model.locations,
                datatypes: model.datatypes,
                identifier: model.identifier,
                privileges: model.privileges,
            },
            expires_in: None,
            key: None,
            flags: model.flags,
        }
    }
}
