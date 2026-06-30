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

use crate::errors::Outcome;
use crate::types::keys::{Crv, Kty, PrivateKey, PublicKey};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PemHelper {
    pem: String,
    crv: Option<Crv>,
    kty: Kty,
}

impl PemHelper {
    pub fn new(pem: String, crv: Option<Crv>, kty: Kty) -> Self {
        Self { pem, crv, kty }
    }

    pub fn priv_from_pem(pem: &str) -> Outcome<Self> {
        let key = PrivateKey::try_from_pkcs8_pem(pem)?;
        Ok(Self {
            pem: pem.to_string(),
            crv: key.crv(),
            kty: key.kty(),
        })
    }

    pub fn pub_from_pem(pem: &str) -> Outcome<Self> {
        let key = PublicKey::try_from_pkcs8_pem(pem)?;
        Ok(Self {
            pem: pem.to_string(),
            crv: key.crv(),
            kty: key.kty(),
        })
    }

    pub fn pem(&self) -> &str {
        &self.pem
    }
    pub fn kty(&self) -> &Kty {
        &self.kty
    }
    pub fn crv(&self) -> Option<&Crv> {
        self.crv.as_ref()
    }
}
