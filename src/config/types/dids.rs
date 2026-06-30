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
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::config::traits::DidConfigTrait;

/// Polymorphic deployment configuration tracking decentralized identifier strategies.
#[derive(Clone, Debug)]
pub enum DidConfig {
    /// Pure cryptographic key derived locally bound scheme.
    Jwk,
    /// Web host infrastructure anchored identifier scheme layout.
    Web { web_config: DidWebConfig },
    /// Fallback variant supporting novel custom experimental schemes.
    Other(String),
}

// ===== PLUGGED SERDE ENGINE COUPLING =============================================================

impl<'de> Deserialize<'de> for DidConfig {
    /// Custom deserializer routing untyped configuration map inputs into structured variants.
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

impl Serialize for DidConfig {
    /// Flattens variant instances into standard config-driven outbound string or object schemas.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            DidConfig::Jwk => serializer.serialize_str("Jwk"),
            DidConfig::Web { web_config } => web_config.serialize(serializer),
            DidConfig::Other(other) => other.serialize(serializer),
        }
    }
}

// ===== SUB-SCHEME LAYOUTS ========================================================================

/// Dedicated parameters driving domain-bound `did:web` deployments.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DidWebConfig {
    /// Target authority domain string host location.
    pub domain: String,
    /// Optional structured deployment directory sub-path matrix.
    pub path: Option<String>,
    /// Custom network port exposure parameter if overriding default TLS hooks.
    pub port: Option<String>,
}

impl DidConfigTrait for DidConfig {
    fn did_config(&self) -> &DidConfig {
        self
    }
}
