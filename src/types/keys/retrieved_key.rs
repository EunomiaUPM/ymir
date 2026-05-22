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

use crate::errors::{BadFormat, Errors, Outcome};
use crate::utils::decode_url_safe_no_pad;
use ed25519_dalek::{Signature as EdSignature, VerifyingKey as EdVerifyingKey};
use rsa::pss::{Signature as RsaSignature, VerifyingKey as RsaVerifyingKey};
use rsa::signature::Verifier;
use rsa::{BigUint, RsaPublicKey};
use serde_json::Value;
use sha2::Sha256;

pub struct RetrievedKey {
    pub did: String,
    pub data: RetrievedKeyData,
}

impl RetrievedKey {
    pub fn verify_bytes(&self, data: &[u8], sig: &[u8]) -> Outcome<()> {
        match &self.data {
            RetrievedKeyData::Rsa { vk } => {
                let signature = RsaSignature::try_from(sig)
                    .map_err(|e| Errors::parse("error parsing RSA signature", Some(Box::new(e))))?;
                vk.verify(data, &signature)
                    .map_err(|e| Errors::forbidden("Invalid Signature", Some(Box::new(e))))
            }
            RetrievedKeyData::Ed25519 { vk } => {
                let signature = EdSignature::try_from(sig).map_err(|e| {
                    Errors::parse("error parsing Ed25519 signature", Some(Box::new(e)))
                })?;
                vk.verify(data, &signature)
                    .map_err(|e| Errors::forbidden("Invalid Signature", Some(Box::new(e))))
            }
        }
    }
}

pub enum RetrievedKeyData {
    Rsa { vk: RsaVerifyingKey<Sha256> },
    Ed25519 { vk: EdVerifyingKey },
}

impl RetrievedKeyData {
    pub fn build_ed25519_data(value: &Value) -> Outcome<RetrievedKeyData> {
        let x_b64 = value["x"]
            .as_str()
            .ok_or_else(|| Errors::format(BadFormat::Received, "JWK Ed25519 missing x", None))?;
        let raw = decode_url_safe_no_pad(x_b64)?;
        let arr: [u8; 32] = raw.as_slice().try_into().map_err(|_| {
            Errors::format(
                BadFormat::Received,
                "Ed25519 public key must be 32 bytes",
                None,
            )
        })?;
        let vk = EdVerifyingKey::from_bytes(&arr)
            .map_err(|e| Errors::forbidden("invalid Ed25519 verifying key", Some(Box::new(e))))?;
        Ok(RetrievedKeyData::Ed25519 { vk })
    }

    pub fn build_rsa_data(jwk: &Value) -> Outcome<RetrievedKeyData> {
        let n_b64 = jwk["n"]
            .as_str()
            .ok_or_else(|| Errors::format(BadFormat::Received, "JWK RSA missing n", None))?;
        let e_b64 = jwk["e"]
            .as_str()
            .ok_or_else(|| Errors::format(BadFormat::Received, "JWK RSA missing e", None))?;
        let n = decode_url_safe_no_pad(n_b64)?;
        let e = decode_url_safe_no_pad(e_b64)?;
        let pub_key = RsaPublicKey::new(BigUint::from_bytes_be(&n), BigUint::from_bytes_be(&e))
            .map_err(|err| Errors::forbidden("invalid RSA public key", Some(Box::new(err))))?;
        let vk = RsaVerifyingKey::<Sha256>::new(pub_key);
        Ok(RetrievedKeyData::Rsa { vk })
    }
}
