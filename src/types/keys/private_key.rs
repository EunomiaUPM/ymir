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
use crate::types::secrets::PemHelper;
use ed25519_dalek::SigningKey as Ed25519SigningKey;
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::SigningKey as PkcsSigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::SigningKey as PssSigningKey;
use rsa::signature::Signer;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use serde_json::Value;
use sha2::{Sha256, Sha384, Sha512};

pub enum PrivateKey {
    Rsa { sk: RsaPrivateKey },
    Ed25519 { sk: Ed25519SigningKey },
}

impl PrivateKey {
    pub fn try_from_pkcs8_pem(pem: &str) -> Outcome<Self> {
        if let Ok(sk) = parse_rsa(pem) {
            return Ok(Self::Rsa { sk });
        }

        if let Ok(sk) = parse_ed25519(pem) {
            return Ok(Self::Ed25519 { sk });
        }

        Err(Errors::format(
            BadFormat::Received,
            "PEM is not a supported Ed25519/RSA PKCS#8",
            None,
        ))
    }

    pub fn from_safe_pem(pem: &str, kty: &Kty, crv: Option<&Crv>) -> Outcome<Self> {
        match (kty, crv) {
            (Kty::Rsa, _) => Ok(PrivateKey::Rsa {
                sk: parse_rsa(pem)?,
            }),
            (Kty::Okp, Some(Crv::Ed25519)) => Ok(PrivateKey::Ed25519 {
                sk: parse_ed25519(pem)?,
            }),
            _ => Err(Errors::not_impl(
                format!("Unsupported key/alg combination: kty={kty}, crv={crv:?}"),
                None,
            )),
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
    pub fn alg(&self) -> Alg {
        match self {
            PrivateKey::Rsa { .. } => Alg::Rs256,
            PrivateKey::Ed25519 { .. } => Alg::EdDsa,
        }
    }

    pub fn cryptosuite(&self) -> Outcome<Cryptosuite> {
        match self {
            Self::Ed25519 { .. } => Ok(Cryptosuite::EddsaJcs2022),
            Self::Rsa { .. } => Err(Errors::not_impl(
                "RSA does not have an active cryptosuite",
                None,
            )),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        match self {
            Self::Rsa { sk: pk } => PublicKey::Rsa {
                vk: pk.to_public_key(),
            },
            Self::Ed25519 { sk: pk } => PublicKey::Ed25519 {
                vk: pk.verifying_key(),
            },
        }
    }

    pub fn public_jwk(&self) -> Value {
        self.public_key().public_jwk()
    }

    pub fn sign_bytes(&self, data: &[u8], alg: Alg) -> Outcome<Vec<u8>> {
        match self {
            PrivateKey::Rsa { sk } => match alg {
                Alg::Rs256 => sign_rs::<Sha256>(sk, data),
                Alg::Rs384 => sign_rs::<Sha384>(sk, data),
                Alg::Rs512 => sign_rs::<Sha512>(sk, data),
                Alg::Ps256 => sign_ps::<Sha256>(sk, data),
                Alg::Ps384 => sign_ps::<Sha384>(sk, data),
                Alg::Ps512 => sign_ps::<Sha512>(sk, data),
                other => Err(Errors::not_impl(
                    format!("Unsupported alg  {}", other),
                    None,
                )),
            },
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
        Self::from_safe_pem(helper.pem(), helper.kty(), helper.crv())
    }
}

fn sign_rs<T>(pk: &RsaPrivateKey, data: &[u8]) -> Outcome<Vec<u8>>
where
    T: rsa::signature::digest::Digest,
{
    let sk = PkcsSigningKey::<T>::from(pk.clone());
    let sig = sk.sign(data);
    Ok(sig.to_bytes().to_vec())
}

fn sign_ps<T>(pk: &RsaPrivateKey, data: &[u8]) -> Outcome<Vec<u8>>
where
    T: rsa::signature::digest::Digest + rsa::signature::digest::FixedOutputReset,
{
    let sk = PssSigningKey::<T>::from(pk.clone());
    let mut rng = rand::thread_rng();
    let sig = sk.sign_with_rng(&mut rng, data);
    Ok(sig.to_bytes().to_vec())
}

fn parse_rsa(pem: &str) -> Outcome<RsaPrivateKey> {
    RsaPrivateKey::from_pkcs8_pem(pem)
        .map_err(|e| Errors::parse("Invalid RSA PKCS#8 PEM", Some(Box::new(e))))
}

fn parse_ed25519(pem: &str) -> Outcome<Ed25519SigningKey> {
    Ed25519SigningKey::from_pkcs8_pem(pem)
        .map_err(|e| Errors::parse("Invalid Ed25519 PKCS#8 PEM", Some(Box::new(e))))
}
