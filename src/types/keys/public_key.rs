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
use ed25519_dalek::{Signature as Ed25519Signature, VerifyingKey as Ed25519VerifyingKey};
use rsa::signature::Verifier;
use sha2::Sha256;
use std::str::FromStr;

use super::jwk::{ed25519_public_key_from_jwk, rsa_public_key_from_jwk};
use crate::types::dids::{VerificationMaterial, VerificationMethod};
use crate::types::keys::{Alg, Crv, Kty};
use rsa::pkcs1v15::{Signature as PkcsSignature, VerifyingKey as PkcsVerifyingKey};
use rsa::pss::{Signature as PssSignature, VerifyingKey as PssVerifyingKey};
use rsa::RsaPublicKey;

pub enum PublicKey {
    RsaRs256 { vk: PkcsVerifyingKey<Sha256> },
    RsaPs256 { vk: PssVerifyingKey<Sha256> },
    Ed25519 { vk: Ed25519VerifyingKey },
}

impl PublicKey {
    pub fn parse_from(vm: &VerificationMethod, alg: &Alg) -> Outcome<PublicKey> {
        let jwk = match &vm.material {
            VerificationMaterial::JsonWebKey { public_key_jwk }
            | VerificationMaterial::JsonWebKey2020 { public_key_jwk } => public_key_jwk,
            VerificationMaterial::Multikey { .. } => {
                return Err(Errors::not_impl(
                    "Multikey verification material not supported",
                    None,
                ));
            }
        };

        let kty_str = jwk
            .get("kty")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Errors::format(BadFormat::Received, "JWK missing 'kty'", None))?;
        let Ok(kty) = Kty::from_str(kty_str);

        let crv = jwk
            .get("crv")
            .and_then(|v| v.as_str())
            .map(|s| {
                let Ok(crv) = Crv::from_str(s);
                crv
            });

        match (&kty, crv.as_ref(), alg) {
            // ── RSA + PKCS#1 v1.5 + SHA-256 ─────────────────────────────
            (Kty::Rsa, _, Alg::Rs256) => {
                let pk = rsa_public_key_from_jwk(jwk)?;
                Ok(PublicKey::RsaRs256 {
                    vk: PkcsVerifyingKey::<Sha256>::new(pk),
                })
            }

            // ── RSA + PSS + SHA-256 ─────────────────────────────────────
            (Kty::Rsa, _, Alg::Ps256) => {
                let pk = rsa_public_key_from_jwk(jwk)?;
                Ok(PublicKey::RsaPs256 {
                    vk: PssVerifyingKey::<Sha256>::new(pk),
                })
            }

            // ── Ed25519 ────────────────────────────────────────────────
            (Kty::Okp, Some(Crv::Ed25519), Alg::EdDsa) => {
                let vk = ed25519_public_key_from_jwk(jwk)?;
                Ok(PublicKey::Ed25519 { vk })
            }

            _ => Err(Errors::not_impl(
                format!("Unsupported key/alg combination: kty={kty}, crv={crv:?}, alg={alg}"),
                None,
            )),
        }
    }

    pub fn verify_bytes(&self, data: &[u8], sig: &[u8]) -> Outcome<()> {
        match &self {
            PublicKey::RsaRs256 { vk } => {
                let signature = PkcsSignature::try_from(sig).map_err(|e| {
                    Errors::format(
                        BadFormat::Received,
                        "invalid PKCS#1 v1.5 signature encoding",
                        Some(Box::new(e)),
                    )
                })?;
                vk.verify(data, &signature)
                    .map_err(|e| Errors::forbidden("Invalid Signature", Some(Box::new(e))))
            }
            PublicKey::RsaPs256 { vk } => {
                let signature = PssSignature::try_from(sig).map_err(|e| {
                    Errors::format(
                        BadFormat::Received,
                        "invalid PSS signature encoding",
                        Some(Box::new(e)),
                    )
                })?;
                vk.verify(data, &signature)
                    .map_err(|e| Errors::forbidden("Invalid Signature", Some(Box::new(e))))
            }
            PublicKey::Ed25519 { vk } => {
                let signature = Ed25519Signature::try_from(sig).map_err(|e| {
                    Errors::format(
                        BadFormat::Received,
                        "invalid Ed25519 signature encoding",
                        Some(Box::new(e)),
                    )
                })?;
                vk.verify(data, &signature)
                    .map_err(|e| Errors::forbidden("Invalid Signature", Some(Box::new(e))))
            }
        }
    }
}

