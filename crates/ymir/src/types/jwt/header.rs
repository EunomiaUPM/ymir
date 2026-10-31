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

use super::JwtType;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::dids::Kid;
use crate::types::keys::Alg;
use super::JwtKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: Alg,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typ: Option<JwtType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cty: Option<String>,
    // pub kid: Kid,
    /// Exactly one of `kid`, `jwk` or `x5c`.
    #[serde(flatten)]
    pub key: JwtKey,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl JwtHeaderTrait for JwtHeader {
    fn header(&self) -> &JwtHeader {
        self
    }
}
pub trait JwtHeaderTrait {
    fn header(&self) -> &JwtHeader;
    fn alg(&self) -> &Alg {
        &self.header().alg
    }
    fn typ(&self) -> Option<&JwtType> {
        self.header().typ.as_ref()
    }
    fn cty(&self) -> Option<&str> {
        self.header().cty.as_deref()
    }
    fn key(&self) -> &JwtKey {
        &self.header().key
    }
    fn kid(&self) -> Outcome<&Kid> {
        self.key().kid()
    }
    fn check_single_key(&self) -> Outcome<()> {
        match ["kid", "jwk", "x5c"].iter().find(|k| self.header().extra.contains_key(**k)) {
            Some(k) => Err(Errors::format(
                BadFormat::Received,
                format!("JWT header names its key twice (also '{k}')"),
                None,
            )),
            None => Ok(()),
        }
    }
}
