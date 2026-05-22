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

use super::{Crv, KeyData, Kty, SerialKey};
use crate::errors::{Errors, Outcome};
use crate::utils::HasId;
use serde_json::Value;
use std::str::FromStr;

pub struct Key {
    id: String,
    data: KeyData,
}

impl HasId for Key {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Key {
    pub fn data(&self) -> &KeyData {
        &self.data
    }
    pub fn public_jwk(&self) -> Value {
        self.data.public_jwk()
    }
    pub fn public_multibase(&self) -> Option<String> {
        self.data.public_multibase()
    }
    pub fn sign_bytes(&self, data: &[u8]) -> Outcome<Vec<u8>> {
        self.data.sign_bytes(data)
    }
    pub fn cryptosuite(&self) -> Outcome<&'static str> {
        self.data.cryptosuite()
    }

    pub fn jws_alg(&self) -> &'static str {
        self.data.jws_alg()
    }
    pub fn new(id: String, data: KeyData) -> Key {
        Key { id, data }
    }
}

impl TryFrom<SerialKey> for Key {
    type Error = Errors;

    fn try_from(value: SerialKey) -> Result<Self, Self::Error> {
        let Ok(kty) = Kty::from_str(&value.kty);

        let crv = value.crv.map(|c| Crv::from_str(&c).unwrap());

        let data = match (&kty, crv.as_ref()) {
            (Kty::Okp, Some(Crv::Ed25519)) => KeyData::build_rsa(&value.pem)?,
            (Kty::Rsa, _) => KeyData::build_ed25519(&value.pem)?,
            _ => {
                let crv = if let Some(crv) = crv {
                    crv.to_string()
                } else {
                    "".to_string()
                };
                return Err(Errors::not_impl(
                    format!("unsupported JWK: kty={} crv={:?}", kty, crv),
                    None,
                ));
            }
        };

        Ok(Key { id: value.id, data })
    }
}
