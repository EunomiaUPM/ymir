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

use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use crate::errors::Outcome;
use super::Cryptosuite;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Alg {
    // HMAC con SHA-2 (simétrico) — RFC 7518 §3.2
    Hs256,
    Hs384,
    Hs512,

    // RSASSA-PKCS1-v1_5 — RFC 7518 §3.3
    Rs256,
    Rs384,
    Rs512,

    // RSASSA-PSS — RFC 7518 §3.5
    Ps256,
    Ps384,
    Ps512,

    // ECDSA — RFC 7518 §3.4 + RFC 8812
    Es256,
    Es384,
    Es512,
    Es256k,

    // EdDSA — RFC 8037 §3.1 (cubre Ed25519 y Ed448)
    EdDsa,

    // Catch-all
    Other(String),
}

impl Alg {
    pub fn from_cryptosuite(suite: &Cryptosuite) -> Self {
        match suite {
            Cryptosuite::EddsaRdfc2022
            | Cryptosuite::EddsaJcs2022 => Alg::EdDsa,

            Cryptosuite::EcdsaRdfc2019
            | Cryptosuite::EcdsaJcs2019 => Alg::Es256,

            Cryptosuite::RsaSignature2018 => Alg::Rs256,

            Cryptosuite::BbsRdfc2023 => Alg::Other("BBS".to_string()),

            Cryptosuite::Other(s) => Alg::Other(s.clone()),
        }
    }
}

impl Display for Alg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Alg::Hs256 => "HS256",
            Alg::Hs384 => "HS384",
            Alg::Hs512 => "HS512",
            Alg::Rs256 => "RS256",
            Alg::Rs384 => "RS384",
            Alg::Rs512 => "RS512",
            Alg::Ps256 => "PS256",
            Alg::Ps384 => "PS384",
            Alg::Ps512 => "PS512",
            Alg::Es256 => "ES256",
            Alg::Es384 => "ES384",
            Alg::Es512 => "ES512",
            Alg::Es256k => "ES256K",
            Alg::EdDsa => "EdDSA",
            Alg::Other(s) => s.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Alg {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "HS256" => Alg::Hs256,
            "HS384" => Alg::Hs384,
            "HS512" => Alg::Hs512,
            "RS256" => Alg::Rs256,
            "RS384" => Alg::Rs384,
            "RS512" => Alg::Rs512,
            "PS256" => Alg::Ps256,
            "PS384" => Alg::Ps384,
            "PS512" => Alg::Ps512,
            "ES256" => Alg::Es256,
            "ES384" => Alg::Es384,
            "ES512" => Alg::Es512,
            "ES256K" => Alg::Es256k,
            "EdDSA" => Alg::EdDsa,
            _ => Alg::Other(s.to_string()),
        })
    }
}

impl_serde_via_str!(Alg);
