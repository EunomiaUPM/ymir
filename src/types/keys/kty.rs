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

use crate::impl_serde_via_str;
use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kty {
    /// Elliptic Curve — RFC 7518 §6.2
    Ec,
    /// RSA — RFC 7518 §6.3
    Rsa,
    /// Octet sequence (claves simétricas, HMAC) — RFC 7518 §6.4
    Oct,
    /// Octet Key Pair (Ed25519/Ed448/X25519/X448) — RFC 8037
    Okp,
    /// Algorithm Key Pair (PQ: ML-DSA, etc.) — draft-ietf-jose-pqc-*
    Akp,
    /// Catch-all para valores desconocidos
    Other(String),
}

impl Display for Kty {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Kty::Ec => "EC",
            Kty::Rsa => "RSA",
            Kty::Oct => "oct",
            Kty::Okp => "OKP",
            Kty::Akp => "AKP",
            Kty::Other(s) => s.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Kty {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "EC" => Kty::Ec,
            "RSA" => Kty::Rsa,
            "oct" => Kty::Oct,
            "OKP" => Kty::Okp,
            "AKP" => Kty::Akp,
            _ => Kty::Other(s.to_string()),
        })
    }
}

impl_serde_via_str!(Kty);
