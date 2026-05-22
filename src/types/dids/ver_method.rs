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

use crate::capabilities::Did;
use crate::types::keys::{Key, KeyData};
use crate::utils::HasId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub controller: String,
    #[serde(flatten)]
    pub material: VerificationMaterial,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VerificationMaterial {
    JsonWebKey {
        #[serde(rename = "publicKeyJwk")]
        public_key_jwk: Value,
    },
    Multikey {
        #[serde(rename = "publicKeyMultibase")]
        public_key_multibase: String,
    },
}

impl VerificationMethod {
    pub fn new(did: &Did, key: &Key) -> Self {
        let controller = did.id().to_string();

        let material = match key.data() {
            KeyData::Ed25519 { .. } => VerificationMaterial::Multikey {
                public_key_multibase: key
                    .public_multibase()
                    .expect("Ed25519 always has multibase"),
            },
            KeyData::Rsa { .. } => VerificationMaterial::JsonWebKey {
                public_key_jwk: key.public_jwk(),
            },
        };

        Self {
            id: key.id().to_string(),
            controller,
            material,
            expires: None,
            revoked: None,
        }
    }
}
