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

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

use crate::types::vcs::VcType;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VerifyReqConfig {
    pub is_cert_allowed: bool,
    #[serde(deserialize_with = "deserialize_vc_type_vec")]
    pub vcs_requested: Vec<VcType>
}

fn deserialize_vc_type_vec<'de, D>(deserializer: D) -> Result<Vec<VcType>, D::Error>
where
    D: Deserializer<'de>
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    strings.into_iter().map(|s| s.parse::<VcType>().map_err(de::Error::custom)).collect()
}
