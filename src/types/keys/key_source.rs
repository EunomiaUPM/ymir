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
use super::{Alg, Certificate, PublicKey};
use crate::errors::Outcome;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub enum KeySource {
    Cert(Certificate),
    PublicKey(PublicKey),
}

impl KeySource {
    pub fn thumbprint(&self) -> String {
        match self {
            KeySource::Cert(cert) => cert.thumbprint_sha256(),
            KeySource::PublicKey(key) => key.jwk_thumbprint(),
        }
    }
    pub fn check_validity(&self) -> Outcome<()> {
        if let KeySource::Cert(cert) = self {
            cert.check_validity()?;
        }
        Ok(())
    }
    pub fn verify_bytes(&self, data: &[u8], sig: &[u8], alg: &Alg) -> Outcome<()> {
        match self {
            KeySource::Cert(cert) => cert.public_key()?.verify_bytes(data, sig, alg),
            KeySource::PublicKey(key) => key.verify_bytes(data, sig, alg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub enum DbKeySource {
    Cert(String),
    PublicKey(Value),
}
