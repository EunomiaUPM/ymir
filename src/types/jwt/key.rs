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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::dids::{Kid};
use crate::types::keys::{Certificate, PublicKey};
use crate::utils::decode_std_b64;

/// The header member naming the signing key (RFC 7515 §4.1); serialized as that member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JwtKey {
    /// A DID URL, resolved through the DID Document.
    Kid(Kid),
    /// The public key itself, as a JWK.
    Jwk(Value),
    /// An X.509 certificate chain, leaf first.
    X5c(Vec<String>),
}

impl JwtKey {
    /// The `kid`, a DID URL.
    pub fn kid(&self) -> Outcome<&Kid> {
        match self {
            JwtKey::Kid(kid) => Ok(kid),
            other => Err(other.not("kid")),
        }
    }

    /// The embedded `jwk`, as a public key; a private one is refused.
    pub fn jwk(&self) -> Outcome<PublicKey> {
        match self {
            JwtKey::Jwk(jwk) => {
                if jwk.get("d").is_some() {
                    return Err(Errors::security("JWT header carries a private key", None));
                }
                PublicKey::parse_from_jwk(jwk)
            }
            other => Err(other.not("jwk")),
        }
    }

    /// The leaf certificate of `x5c`, only parsed: validating the chain is the caller's job.
    pub fn cert(&self) -> Outcome<Certificate> {
        match self {
            JwtKey::X5c(chain) => {
                let leaf = chain.first().ok_or_else(|| Errors::format(BadFormat::Received, "empty x5c", None))?;
                Certificate::try_from_der(&decode_std_b64(leaf)?)     // no existe aún: hoy solo hay PEM
            }
            other => Err(other.not("x5c")),
        }
    }

    fn not(&self, expected: &str) -> Errors {
        let got = match self {
            JwtKey::Kid(_) => "kid",
            JwtKey::Jwk(_) => "jwk",
            JwtKey::X5c(_) => "x5c"
        };
        Errors::security(format!("JWT header names its key as '{got}', '{expected}' expected"), None)
    }
}