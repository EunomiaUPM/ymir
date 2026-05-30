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

use super::{Alg, Crv, Cryptosuite, Kty, PublicKey};
use crate::errors::{BadFormat, Errors, Outcome};
use super::jwk::{ed25519_public_jwk, rsa_public_jwk};
use ed25519_dalek::{SigningKey as Ed25519SigningKey};
use rsa::pkcs8::{DecodePrivateKey};
use rsa::pss::{SigningKey as PssSigningKey};
use rsa::pkcs1v15::{SigningKey as PkcsSigningKey};
use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding};
use rsa::signature::Signer;
use serde_json::Value;
use sha2::Sha256;
use crate::types::secrets::PemHelper;

pub enum PrivateKey {
    RsaRs256 {
        sk: PkcsSigningKey<Sha256>,
    },
    RsaPs256 {
        sk: PssSigningKey<Sha256>,
    },
    Ed25519 {
        sk: Ed25519SigningKey,
    },
}

impl PrivateKey {
    pub fn from_pkcs8_pem(pem: &str) -> Outcome<Self> {
        if let Ok(sk) = Ed25519SigningKey::from_pkcs8_pem(pem) {
            return Ok(PrivateKey::Ed25519 { sk });
        }
        if let Ok(key) = rsa::RsaPrivateKey::from_pkcs8_pem(pem) {
            return Ok(PrivateKey::RsaRs256 { sk: PkcsSigningKey::<Sha256>::new(key) });
        }
        Err(Errors::format(BadFormat::Received,
                           "PEM is not a supported Ed25519/RSA PKCS#8", None))
    }

    pub fn try_from_pkcs8_pem(
        pem: &str,
        kty: &Kty,
        crv: Option<&Crv>,
        alg: &Alg,
    ) -> Outcome<Self> {
        match (kty, crv, alg) {
            (Kty::Rsa, _, Alg::Rs256) => Ok(Self::RsaRs256 {
                sk: PkcsSigningKey::<Sha256>::new(rsa_priv_from_pkcs8_pem(pem)?),
            }),

            (Kty::Rsa, _, Alg::Ps256) => Ok(Self::RsaPs256 {
                sk: PssSigningKey::<Sha256>::new(rsa_priv_from_pkcs8_pem(pem)?),
            }),

            (Kty::Okp, Some(Crv::Ed25519), Alg::EdDsa) => {
                let sk = Ed25519SigningKey::from_pkcs8_pem(pem).map_err(|e| {
                    Errors::format(
                        BadFormat::Received,
                        "invalid Ed25519 PKCS#8 PEM",
                        Some(Box::new(e)),
                    )
                })?;
                Ok(Self::Ed25519 { sk })
            }

            _ => Err(Errors::not_impl(
                format!(
                    "Unsupported key/alg combination: kty={kty}, crv={crv:?}, alg={alg}"
                ),
                None,
            )),
        }
    }
    pub fn kty(&self) -> Kty {
        match self {
            Self::RsaRs256 { .. } | Self::RsaPs256 { .. } => Kty::Rsa,
            Self::Ed25519 { .. } => Kty::Okp,
        }
    }

    pub fn crv(&self) -> Option<Crv> {
        match self {
            Self::RsaRs256 { .. } | Self::RsaPs256 { .. } => None,
            Self::Ed25519 { .. } => Some(Crv::Ed25519),
        }
    }

    pub fn alg(&self) -> Alg {
        match self {
            Self::RsaRs256 { .. } => Alg::Rs256,
            Self::RsaPs256 { .. } => Alg::Ps256,
            Self::Ed25519 { .. } => Alg::EdDsa,
        }
    }
    pub fn cryptosuite(&self) -> Outcome<Cryptosuite> {
        match self {
            Self::Ed25519 { .. } => Ok(Cryptosuite::EddsaJcs2022),
            Self::RsaRs256 { .. } | Self::RsaPs256 { .. } => Err(Errors::not_impl(
                "RSA no tiene cryptosuite de Data Integrity; usa firma JWT (enveloped)",
                None,
            )),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        match self {
            Self::RsaRs256 { sk } => PublicKey::RsaRs256 { vk: sk.verifying_key() },
            Self::RsaPs256 { sk } => PublicKey::RsaPs256 { vk: sk.verifying_key() },
            Self::Ed25519 { sk } => PublicKey::Ed25519 { vk: sk.verifying_key() },
        }
    }

    pub fn public_jwk(&self) -> Value {
        match self {
            Self::Ed25519 { sk } => {
                ed25519_public_jwk(&sk.verifying_key())
            }
            Self::RsaRs256 { sk } => rsa_public_jwk(sk.verifying_key().as_ref()),
            Self::RsaPs256 { sk } => rsa_public_jwk(sk.verifying_key().as_ref()),
        }
    }

    pub fn sign_bytes(&self, data: &[u8]) -> Outcome<Vec<u8>> {
        match self {
            PrivateKey::RsaRs256 { sk } => {
                let sig = sk.sign(data);
                Ok(sig.to_bytes().to_vec())
            }
            PrivateKey::RsaPs256 { sk } => {
                let mut rng = rand::thread_rng();
                let sig = sk.sign_with_rng(&mut rng, data);
                Ok(sig.to_bytes().to_vec())
            }
            PrivateKey::Ed25519 { sk } => {
                let sig = sk.sign(data);
                Ok(sig.to_bytes().to_vec())
            }
        }
    }
}

impl TryFrom<PemHelper> for PrivateKey {
    type Error = Errors;

    fn try_from(helper: PemHelper) -> Result<Self, Self::Error> {
        Self::try_from_pkcs8_pem(helper.pem(), helper.kty(), helper.crv(), helper.alg())
    }
}

fn rsa_priv_from_pkcs8_pem(pem: &str) -> Outcome<rsa::RsaPrivateKey> {
    rsa::RsaPrivateKey::from_pkcs8_pem(pem).map_err(|e| {
        Errors::format(BadFormat::Received, "invalid RSA PKCS#8 PEM", Some(Box::new(e)))
    })
}


