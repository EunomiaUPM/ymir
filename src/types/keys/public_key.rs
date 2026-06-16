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
use crate::types::dids::{VerificationMaterial, VerificationMethod};
use crate::types::keys::{Alg, Crv, Kty};
use crate::types::secrets::PemHelper;
use crate::utils::{decode_url_safe_no_pad, encode_url_safe_no_pad};
use ed25519_dalek::{Signature as Ed25519Signature, VerifyingKey as Ed25519VerifyingKey};
use rsa::pkcs1v15::{Signature as PkcsSignature, VerifyingKey as PkcsVerifyingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::pss::{Signature as PssSignature, VerifyingKey as PssVerifyingKey};
use rsa::signature::Verifier;
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, RsaPublicKey};
use serde_json::{Value, json};
use sha2::{Sha256, Sha384, Sha512};
use std::str::FromStr;
use x509_parser::pem::parse_x509_pem;
use x509_parser::prelude::*;

pub enum PublicKey {
    Rsa { vk: RsaPublicKey },
    Ed25519 { vk: Ed25519VerifyingKey },
}

impl PublicKey {
    pub fn try_from_pkcs8_pem(pem: &str) -> Outcome<Self> {
        if let Ok(vk) = parse_rsa_pem(pem) {
            return Ok(PublicKey::Rsa { vk });
        }
        if let Ok(vk) = parse_ed25519_pem(pem) {
            return Ok(PublicKey::Ed25519 { vk });
        }

        Err(Errors::format(
            BadFormat::Received,
            "PEM is not a supported Ed25519/RSA PKCS#8",
            None,
        ))
    }
    pub fn try_from_pkcs8_der(der: &[u8]) -> Outcome<Self> {
        if let Ok(vk) = parse_rsa_der(der) {
            return Ok(PublicKey::Rsa { vk });
        }
        if let Ok(vk) = parse_ed25519_der(der) {
            return Ok(PublicKey::Ed25519 { vk });
        }

        Err(Errors::format(
            BadFormat::Received,
            "PEM is not a supported Ed25519/RSA PKCS#8",
            None,
        ))
    }

    pub fn from_safe_pem(pem: &str, kty: &Kty, crv: Option<&Crv>) -> Outcome<Self> {
        match (kty, crv) {
            (Kty::Rsa, _) => Ok(PublicKey::Rsa {
                vk: parse_rsa_pem(pem)?,
            }),
            (Kty::Okp, Some(Crv::Ed25519)) => Ok(PublicKey::Ed25519 {
                vk: parse_ed25519_pem(pem)?,
            }),
            _ => Err(Errors::not_impl(
                format!("Unsupported key/alg combination: kty={kty}, crv={crv:?}"),
                None,
            )),
        }
    }

    pub fn try_from_certificate_pem(cert_pem: &str) -> Outcome<Self> {
        let (_, pem) = parse_x509_pem(cert_pem.as_bytes()).map_err(|e| {
            Errors::parse("Error parsing certificate", Some(Box::new(e)))
        })?;

        let (_, cert) = X509Certificate::from_der(&pem.contents).map_err(|e| {
            Errors::parse("Invalid Certificate Structure", Some(Box::new(e)))
        })?;

        Self::try_from_pkcs8_der(cert.public_key().raw)
    }
    
    pub fn parse_from(vm: &VerificationMethod) -> Outcome<PublicKey> {
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

        let crv = jwk.get("crv").and_then(|v| v.as_str()).map(|s| {
            let Ok(crv) = Crv::from_str(s);
            crv
        });

        match (&kty, crv.as_ref()) {
            (Kty::Rsa, _) => {
                let vk = rsa_public_key_from_jwk(jwk)?;
                Ok(PublicKey::Rsa { vk })
            }

            (Kty::Okp, Some(Crv::Ed25519)) => {
                let vk = ed25519_public_key_from_jwk(jwk)?;
                Ok(PublicKey::Ed25519 { vk })
            }

            _ => Err(Errors::not_impl(
                format!("Unsupported key/alg combination: kty={kty}, crv={crv:?}"),
                None,
            )),
        }
    }

    pub fn verify_bytes(&self, data: &[u8], sig: &[u8], alg: &Alg) -> Outcome<()> {
        match self {
            PublicKey::Rsa { vk } => match alg {
                Alg::Rs256 => verify_rs::<Sha256>(vk, data, sig),
                Alg::Rs384 => verify_rs::<Sha384>(vk, data, sig),
                Alg::Rs512 => verify_rs::<Sha512>(vk, data, sig),
                Alg::Ps256 => verify_ps::<Sha256>(vk, data, sig),
                Alg::Ps384 => verify_ps::<Sha384>(vk, data, sig),
                Alg::Ps512 => verify_ps::<Sha512>(vk, data, sig),
                other => Err(Errors::not_impl(
                    format!("Unsupported alg  {}", other),
                    None,
                )),
            },
            PublicKey::Ed25519 { vk: pk } => {
                let signature = Ed25519Signature::try_from(sig).map_err(|e| {
                    Errors::format(
                        BadFormat::Received,
                        "invalid Ed25519 signature encoding",
                        Some(Box::new(e)),
                    )
                })?;
                pk.verify(data, &signature)
                    .map_err(|e| Errors::forbidden("Invalid Signature", Some(Box::new(e))))
            }
        }
    }
    pub fn kty(&self) -> Kty {
        match self {
            Self::Rsa { .. } => Kty::Rsa,
            Self::Ed25519 { .. } => Kty::Okp,
        }
    }

    pub fn crv(&self) -> Option<Crv> {
        match self {
            Self::Rsa { .. } => None,
            Self::Ed25519 { .. } => Some(Crv::Ed25519),
        }
    }
    pub fn public_jwk(&self) -> Value {
        match self {
            PublicKey::Rsa { vk } => {
                json!({
                    "kty": "RSA",
                    "n": encode_url_safe_no_pad(vk.n().to_bytes_be()),
                    "e": encode_url_safe_no_pad(vk.e().to_bytes_be()),
                })
            }
            PublicKey::Ed25519 { vk } => {
                json!({
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "x": encode_url_safe_no_pad(vk.to_bytes()),
                })
            }
        }
    }
}

impl TryFrom<PemHelper> for PublicKey {
    type Error = Errors;

    fn try_from(helper: PemHelper) -> Result<Self, Self::Error> {
        Self::from_safe_pem(helper.pem(), helper.kty(), helper.crv())
    }
}

fn verify_rs<T>(pk: &RsaPublicKey, data: &[u8], sig: &[u8]) -> Outcome<()>
where
    T: rsa::signature::digest::Digest,
{
    let vk = PkcsVerifyingKey::<T>::from(pk.clone());

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

fn verify_ps<T>(pk: &RsaPublicKey, data: &[u8], sig: &[u8]) -> Outcome<()>
where
    T: rsa::signature::digest::Digest + rsa::signature::digest::FixedOutputReset,
{
    let vk = PssVerifyingKey::<T>::from(pk.clone());

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

fn parse_rsa_pem(pem: &str) -> Outcome<RsaPublicKey> {
    RsaPublicKey::from_public_key_pem(pem)
        .map_err(|e| Errors::parse("Invalid RSA PKCS#8 PEM", Some(Box::new(e))))
}

fn parse_ed25519_pem(pem: &str) -> Outcome<Ed25519VerifyingKey> {
    Ed25519VerifyingKey::from_public_key_pem(pem)
        .map_err(|e| Errors::parse("Invalid Ed25519 PKCS#8 PEM", Some(Box::new(e))))
}

fn parse_rsa_der(der: &[u8]) -> Outcome<RsaPublicKey> {
    RsaPublicKey::from_public_key_der(der)
        .map_err(|e| Errors::parse("Invalid RSA PKCS#8 DER", Some(Box::new(e))))
}

fn parse_ed25519_der(der: &[u8]) -> Outcome<Ed25519VerifyingKey> {
    Ed25519VerifyingKey::from_public_key_der(der)
        .map_err(|e| Errors::parse("Invalid Ed25519 PKCS#8 DER", Some(Box::new(e))))
}

pub fn rsa_public_key_from_jwk(jwk: &Value) -> Outcome<RsaPublicKey> {
    let n_b64 = jwk
        .get("n")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Errors::format(BadFormat::Received, "RSA JWK missing 'n'", None))?;
    let e_b64 = jwk
        .get("e")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Errors::format(BadFormat::Received, "RSA JWK missing 'e'", None))?;

    let n_bytes = decode_url_safe_no_pad(n_b64)?;
    let e_bytes = decode_url_safe_no_pad(e_b64)?;

    let n = BigUint::from_bytes_be(&n_bytes);
    let e = BigUint::from_bytes_be(&e_bytes);

    RsaPublicKey::new(n, e).map_err(|err| {
        Errors::format(
            BadFormat::Received,
            "Invalid RSA public key components",
            Some(Box::new(err)),
        )
    })
}

pub fn ed25519_public_key_from_jwk(jwk: &Value) -> Outcome<Ed25519VerifyingKey> {
    let x_b64 = jwk
        .get("x")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Errors::format(BadFormat::Received, "OKP JWK missing 'x'", None))?;

    let x_bytes = decode_url_safe_no_pad(x_b64)?;

    let arr: [u8; 32] = x_bytes.as_slice().try_into().map_err(|_| {
        Errors::format(
            BadFormat::Received,
            format!("Ed25519 public key must be 32 bytes, got {}", x_bytes.len()),
            None,
        )
    })?;

    Ed25519VerifyingKey::from_bytes(&arr).map_err(|err| {
        Errors::format(
            BadFormat::Received,
            "Invalid Ed25519 public key bytes",
            Some(Box::new(err)),
        )
    })
}
