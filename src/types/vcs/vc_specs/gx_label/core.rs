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
pub struct GxLabelCredSubject {
    pub id: String,
    #[serde(rename = "gx:labelLevel")]
    pub label_level: String,
    #[serde(rename = "gx:engine_version")]
    pub engine_version: String,
    #[serde(rename = "gx:rules_version")]
    pub rules_version: String,
    #[serde(rename = "gx:compliant_credentials")]
    pub compliant_credentials: Vec<CompliantCredential>,
    #[serde(rename = "gx:validated_criteria")]
    pub validated_criteria: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompliantCredential {
    pub id: String,
    pub r#type: String,
    #[serde(rename = "gx:digestSRI")]
    pub digest_sri: String,
}

