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
use rsa::signature::Verifier;
use rsa::{BigUint, RsaPublicKey};
use serde_json::Value;
use sha2::Sha256;

// Importamos los verificadores y firmas de ambos esquemas
use rsa::pss::{Signature as PssSignature, VerifyingKey as PssVerifyingKey};
use rsa::pkcs1v15::{Signature as PkcsSignature, VerifyingKey as PkcsVerifyingKey};

pub struct RetrievedKey {
    pub did: String,
    pub data: RetrievedKeyData,
}

impl RetrievedKey {
    pub fn verify_bytes(&self, data: &[u8], sig: &[u8]) -> Outcome<()> {
        match &self.data {
            RetrievedKeyData::Rsa { vk } => {
                vk.verify(data, sig)
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

// Estructura que guarda ambos esquemas RSA precomputados
pub struct RsaScheme {
    pss_vk: PssVerifyingKey<Sha256>,
    pkcs_vk: PkcsVerifyingKey<Sha256>,
}

impl RsaScheme {
    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<(), rsa::signature::Error> {
        // 1. Intentamos verificar usando PKCS#1 v1.5 (ej. RS256, el más común)
        if let Ok(pkcs_sig) = PkcsSignature::try_from(sig) {
            if self.pkcs_vk.verify(msg, &pkcs_sig).is_ok() {
                return Ok(());
            }
        }

        // 2. Si falla o no se pudo parsear como PKCS#1 v1.5, intentamos con PSS (ej. PS256)
        let pss_sig = PssSignature::try_from(sig)?;
        self.pss_vk.verify(msg, &pss_sig)
    }
}

pub enum RetrievedKeyData {
    // vk es de tipo RsaScheme, lo que mantiene la compatibilidad
    Rsa { vk: RsaScheme },
    Ed25519 { vk: EdVerifyingKey },
}

impl RetrievedKeyData {
    pub fn build_ed25519_data(value: &Value) -> Outcome<RetrievedKeyData> {
        // 1. Validamos que el tipo de clave sea OKP (Octet Key Pair)
        if let Some(kty) = value["kty"].as_str() {
            if kty != "OKP" {
                return Err(Errors::format(
                    BadFormat::Received,
                    "JWK kty is not OKP for Ed25519",
                    None
                ));
            }
        }

        // 2. Validamos la curva (crv) de forma obligatoria
        let crv = value["crv"]
            .as_str()
            .ok_or_else(|| Errors::format(BadFormat::Received, "JWK Ed25519 missing crv", None))?;

        if crv != "Ed25519" {
            return Err(Errors::format(
                BadFormat::Received,
                format!("Unsupported OKP curve: {}", crv),
                None,
            ));
        }

        // 4. Decodificamos y construimos la clave
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
        // Validamos el tipo de clave
        if let Some(kty) = jwk["kty"].as_str() {
            if kty != "RSA" {
                return Err(Errors::format(BadFormat::Received, "JWK kty is not RSA", None));
            }
        }

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

        // Inicializamos ambos verificadores RSA
        let pss_vk = PssVerifyingKey::<Sha256>::new(pub_key.clone());
        let pkcs_vk = PkcsVerifyingKey::<Sha256>::new(pub_key);

        let vk = RsaScheme { pss_vk, pkcs_vk };

        Ok(RetrievedKeyData::Rsa { vk })
    }
}