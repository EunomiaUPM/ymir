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

//! Verifiable Presentation: the holder's wrapper around the credentials it presents. Its
//! credentials are JWT strings in VC DM 1.1 and enveloped objects in VC DM 2.0 (§4.13).

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod jwt;

pub use jwt::{VpJwtClaimsBuilder, VpPayload, VpPayloadV1};

use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::vcs::W3cDataModelVersion;
use crate::utils::OneOrMany;

/// Type of a credential carried as a `data:` URL (VC DM 2.0 §4.13.1).
pub const ENVELOPED_VC_TYPE: &str = "EnvelopedVerifiableCredential";
/// `data:` URL prefixes of a JWT-secured VC: VC-JOSE-COSE, then Gaia-X 24.07's.
pub const VC_JWT_DATA_URL_PREFIXES: [&str; 2] =
    ["data:application/vc+jwt,", "data:application/vc+ld+json+jwt,"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VpDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    /// Optional in VC DM 2.0; in 1.1 JWTs it goes in `jti`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    // pub r#type: Vec<String>,
    /// One or more types (VC DM §4.13): a string or an array.
    pub r#type: OneOrMany<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
    #[serde(rename = "verifiableCredential")]
    // pub verifiable_credential: Vec<String>,
    // pub verifiable_credential: Vec<PresentedCredential>,
    /// One or more credentials: a single one may come without an array.
    pub verifiable_credential: OneOrMany<PresentedCredential>,
}

/// A credential inside a presentation.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum PresentedCredential {
    /// VC DM 1.1: the JWT itself.
    Jwt(String),
    /// VC DM 2.0: an `EnvelopedVerifiableCredential` holding the JWT in a `data:` URL.
    Enveloped(EnvelopedCredential),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EnvelopedCredential {
    /// A string or an array of them.
    #[serde(rename = "@context")]
    pub context: Value,
    pub r#type: String,
    pub id: String,
}

impl VpDocument {
    /// The JWTs of its credentials.
    pub fn credential_jwts(&self) -> Outcome<Vec<String>> {
        self.verifiable_credential
            .iter()
            .map(|c| c.jwt().map(str::to_string))
            .collect()
    }
}

impl PresentedCredential {
    /// `jwt` as the given data model presents it.
    pub fn new(jwt: impl Into<String>, version: W3cDataModelVersion) -> Self {
        let jwt = jwt.into();
        match version {
            W3cDataModelVersion::V1 => PresentedCredential::Jwt(jwt),
            W3cDataModelVersion::V2 => PresentedCredential::Enveloped(EnvelopedCredential {
                context: Value::from(version.context()),
                r#type: ENVELOPED_VC_TYPE.to_string(),
                id: format!("{}{}", VC_JWT_DATA_URL_PREFIXES[0], jwt),
            }),
        }
    }

    /// The credential's JWT.
    pub fn jwt(&self) -> Outcome<&str> {
        match self {
            PresentedCredential::Jwt(jwt) => Ok(jwt),
            PresentedCredential::Enveloped(env) => {
                if env.r#type != ENVELOPED_VC_TYPE {
                    return Err(Errors::format(
                        BadFormat::Received,
                        format!("Presented credential of type '{}'", env.r#type),
                        None,
                    ));
                }
                VC_JWT_DATA_URL_PREFIXES
                    .iter()
                    .find_map(|prefix| env.id.strip_prefix(prefix))
                    .ok_or_else(|| {
                        Errors::format(
                            BadFormat::Received,
                            "Enveloped credential is not a JWT data: URL",
                            None,
                        )
                    })
            }
        }
    }
}
