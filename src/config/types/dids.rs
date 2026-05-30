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

use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::config::traits::DidConfigTrait;

#[derive(Clone, Debug)]
pub enum DidConfig {
    Jwk,
    Web {
        web_config: DidWebConfig,
    },
    Other(String),
}

impl<'de> Deserialize<'de> for DidConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let tag = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| <D::Error as Error>::missing_field("type"))?;

        Ok(match tag {
            "Jwk" => DidConfig::Jwk,
            "Web" => {
                let web_config =
                    DidWebConfig::deserialize(&value).map_err(<D::Error as Error>::custom)?;
                DidConfig::Web { web_config }
            }
            other => DidConfig::Other(other.to_string()),
        })
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct DidWebConfig {
    pub domain: String,
    pub path: Option<String>,
    pub port: Option<String>,
}

impl DidConfigTrait for DidConfig {
    fn did_config(&self) -> &DidConfig {
        self
    }
}


