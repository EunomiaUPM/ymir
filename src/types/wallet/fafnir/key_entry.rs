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

use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::errors::Errors;
use crate::types::keys::{Crv, Key, KeyData, Kty};
use crate::utils::HasId;

#[derive(Serialize, Deserialize)]
pub struct KeyEntryReq {
    pub alias: String,
    pub kty: Kty,
    pub crv: Option<Crv>,
    pub pem: String,
}

#[derive(Serialize, Deserialize)]
pub struct KeyEntry {
    pub id: String,
    pub alias: String,
    pub kty: Kty,
    pub crv: Option<Crv>,
    pub pem: String,
}

impl HasId for KeyEntry {
    fn id(&self) -> &str {
        &self.id
    }
}

impl TryInto<Key> for KeyEntry {
    type Error = Errors;

    fn try_into(self) -> Result<Key, Self::Error> {
        let data = match (&self.kty, self.crv.as_ref()) {
            (Kty::Okp, Some(Crv::Ed25519)) => {
                KeyData::build_ed25519(&self.pem)?
            }
            (Kty::Rsa, _) => {
                KeyData::build_rsa(&self.pem)?
            }
            _ => {
                let crv = if let Some(crv) = &self.crv.as_ref() {
                    crv.to_string()
                } else {
                    "".to_string()
                };
                return Err(Errors::not_impl(
                    format!("unsupported JWK: kty={} crv={:?}", &self.kty, crv),
                    None,
                ));
            }
        };

        Ok(Key::new(self.id, data))
    }
}