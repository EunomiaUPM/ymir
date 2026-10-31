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

use crate::{impl_seaorm_via_str, impl_serde_via_str};
use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Crv {
    // EC family
    P256,
    P384,
    P521,
    Secp256k1,
    // OKP family — firma
    Ed25519,
    Ed448,
    // OKP family — key agreement (ECDH-like)
    X25519,
    X448,
    // Catch-all
    Other(String),
}

impl Display for Crv {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Crv::P256 => "P-256",
            Crv::P384 => "P-384",
            Crv::P521 => "P-521",
            Crv::Secp256k1 => "secp256k1",
            Crv::Ed25519 => "Ed25519",
            Crv::Ed448 => "Ed448",
            Crv::X25519 => "X25519",
            Crv::X448 => "X448",
            Crv::Other(s) => s.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Crv {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "P-256" => Crv::P256,
            "P-384" => Crv::P384,
            "P-521" => Crv::P521,
            "secp256k1" => Crv::Secp256k1,
            "Ed25519" => Crv::Ed25519,
            "Ed448" => Crv::Ed448,
            "X25519" => Crv::X25519,
            "X448" => Crv::X448,
            _ => Crv::Other(s.to_string()),
        })
    }
}

impl_serde_via_str!(Crv);
impl_seaorm_via_str!(Crv, 32);
