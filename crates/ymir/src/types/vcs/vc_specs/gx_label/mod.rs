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
use serde_json::Value;

use crate::errors::Outcome;
use crate::types::crypto::{Canon, HashAlg};
use crate::types::jwt::Jwt;

/// Subject of a `gx:LabelCredential`, or of a `gx:ParticipantComplianceCredential` without a level.
#[derive(Debug, Serialize, Deserialize)]
pub struct GxLabelCredSubject {
    pub id: String,
    #[serde(rename = "gx:labelLevel", default, skip_serializing_if = "Option::is_none")]
    pub label_level: Option<String>,
    #[serde(rename = "gx:engineVersion")]
    pub engine_version: String,
    #[serde(rename = "gx:rulesVersion")]
    pub rules_version: String,
    #[serde(rename = "gx:compliantCredentials")]
    pub compliant_credentials: Vec<CompliantCredential>,
    #[serde(rename = "gx:validatedCriteria")]
    pub validated_criteria: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompliantCredential {
    /// The `id` of the checked credential.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "gx:credentialType")]
    pub credential_type: String,
    #[serde(rename = "digestSRI")]
    pub digest_sri: String,
}

impl CompliantCredential {
    /// Gaia-X `digestSRI` of a credential JWT: sha256, in hex, of its canonical payload.
    pub fn digest_of(jwt: &str) -> Outcome<String> {
        // The whole payload, untyped, so the digest covers every claim.
        let jwt: Jwt<Value> = Jwt::parse(jwt)?;
        let payload = serde_json::to_value(jwt.unverified_claims())?;
        let alg = HashAlg::Sha256;
        let hex: String = Canon::new(&payload)?
            .digest(alg)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Ok(format!("{}-{}", alg.label(), hex))
    }
}
