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
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Client4GR {
    pub key: Key4GR,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Key4GR {
    pub proof: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwk: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cert: Option<String>,
}
