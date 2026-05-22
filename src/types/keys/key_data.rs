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

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::{Signer, SigningKey as EdSigningKey, VerifyingKey as EdVerifyingKey};
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::{SigningKey as RsaSigningKey, VerifyingKey as RsaVerifyingKey};
use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding};
use rsa::traits::PublicKeyParts;
use serde_json::{json, Value};
use sha2::Sha256;
use crate::errors::{BadFormat, Errors, Outcome};
use super::{Kty, Crv, SerialKey, Key};


pub enum KeyData {
    Rsa {
        sk: RsaSigningKey<Sha256>,
        vk: RsaVerifyingKey<Sha256>,
    },
    Ed25519 {
        sk: EdSigningKey,
        vk: EdVerifyingKey,
    },
}

impl KeyData {
    pub fn kty(&self) -> Kty {
        match self {
            KeyData::Rsa { .. } => Kty::Rsa,
            KeyData::Ed25519 { .. } => Kty::Okp,
        }
    }

    pub fn crv(&self) -> Option<Crv> {
        match self {
            KeyData::Rsa { .. } => None,
            KeyData::Ed25519 { .. } => Some(Crv::Ed25519),
        }
    }
    pub fn public_jwk(&self) -> Value {
        match &self {
            KeyData::Ed25519 { vk, .. } => json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": URL_SAFE_NO_PAD.encode(vk.to_bytes()),
            }),
            KeyData::Rsa { vk, .. } => {
                let pk = vk.as_ref();
                json!({
                    "kty": "RSA",
                    "n": URL_SAFE_NO_PAD.encode(pk.n().to_bytes_be()),
                    "e": URL_SAFE_NO_PAD.encode(pk.e().to_bytes_be()),
                })
            }
        }
    }
    pub fn public_multibase(&self) -> Option<String> {
        match &self {
            KeyData::Ed25519 { vk, .. } => {
                let mut bytes = vec![0xed, 0x01];
                bytes.extend_from_slice(&vk.to_bytes());
                Some(format!("z{}", bs58::encode(bytes).into_string()))
            }
            KeyData::Rsa { .. } => None,
        }
    }
    pub fn sign_bytes(&self, data: &[u8]) -> Outcome<Vec<u8>> {
        match &self {
            KeyData::Rsa { sk, .. } => {
                let mut rng = rand::thread_rng();
                let sig = sk.sign_with_rng(&mut rng, data);
                Ok(sig.to_bytes().to_vec())
            }
            KeyData::Ed25519 { sk, .. } => {
                let sig = sk.sign(data);
                Ok(sig.to_bytes().to_vec())
            }
        }
    }
    pub fn build_rsa(pem: &str) -> Outcome<Self> {
        let key = rsa::RsaPrivateKey::from_pkcs8_pem(pem)
            .map_err(|e| Errors::format(BadFormat::Received, "invalid RSA PKCS#8 PEM", Some(Box::new(e))))?;
        let sk: RsaSigningKey<Sha256> = RsaSigningKey::from(key);
        let vk = sk.verifying_key();
        Ok(KeyData::Rsa { sk, vk })
    }
    pub fn build_ed25519(pem: &str) -> Outcome<Self> {
        let sk = EdSigningKey::from_pkcs8_pem(pem)
            .map_err(|e| Errors::format(BadFormat::Received, "invalid Ed25519 PKCS#8 PEM", Some(Box::new(e))))?;
        let vk = sk.verifying_key();

        Ok(KeyData::Ed25519 { sk, vk })
    }

    pub fn to_did_jwk(&self) -> Outcome<String> {
        let jwk = self.public_jwk();
        let jwk_json = serde_json::to_string(&jwk)?;
        let encoded = URL_SAFE_NO_PAD.encode(jwk_json.as_bytes());
        Ok(format!("did:jwk:{encoded}"))
    }

    pub fn cryptosuite(&self) -> Outcome<&'static str> {
        match self {
            KeyData::Ed25519 { .. } => Ok("eddsa-jcs-2022"),
            KeyData::Rsa { .. } => Err(Errors::format(
                BadFormat::Received,
                "RSA has no maintained Data Integrity cryptosuite — use sign_enveloped",
                None,
            )),
        }
    }

    pub fn jws_alg(&self) -> &'static str {
        match self {
            KeyData::Ed25519 { .. } => "EdDSA",
            KeyData::Rsa { .. } => "PS256",
        }
    }
}
