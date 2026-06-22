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

use super::HasId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct VcPlan {
    pub id: String,
    pub r#type: VcBodyType,
}

#[derive(Serialize, Deserialize)]
pub struct VcModel {
    pub id: String,
    pub r#type: VcBodyType,
    #[serde(rename = "parsedDocument")]
    pub parsed_document: Value,
    #[serde(rename = "addedOn", default, skip_serializing_if = "Option::is_none")]
    pub added_on: Option<DateTime<Utc>>,
}

impl Into<VcModel> for VcPlan {
    fn into(self) -> VcModel {
        todo!()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum VcBodyType {
    Jwt(String),
    Value(Value),
}

impl HasId for VcModel {
    fn id(&self) -> &str {
        &self.id
    }
}
