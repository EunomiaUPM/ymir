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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crate::impl_serde_via_str;

/// The `typ` header of a JWT: read case-insensitively, `application/` optional (RFC 7515 §4.1.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JwtType {
    Jwt,
    Vp,
    VpLd,
    VcLd,
    Vc,
    Oid4vciProof,
    RequestObject,
    /// Any other value, as received.
    Other(String),
}

impl Display for JwtType {
    /// Writes the registered form, which is what other implementations compare literally.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            JwtType::Jwt => "JWT",
            JwtType::Vp => "vp+jwt",
            JwtType::VpLd => "vp+ld+jwt",
            JwtType::VcLd => "vc+ld+json+jwt",
            JwtType::Vc => "vc+jwt",
            JwtType::Oid4vciProof => "openid4vci-proof+jwt",
            JwtType::RequestObject => "oauth-authz-req+jwt",
            JwtType::Other(s) => s.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for JwtType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();
        Ok(match lower.strip_prefix("application/").unwrap_or(&lower) {
            "jwt" => JwtType::Jwt,
            "vp+jwt" => JwtType::Vp,
            "vp+ld+jwt" => JwtType::VpLd,
            "vc+ld+json+jwt" => JwtType::VcLd,
            "vc+jwt" => JwtType::Vc,
            "openid4vci-proof+jwt" => JwtType::Oid4vciProof,
            "oauth-authz-req+jwt" => JwtType::RequestObject,
            _ => JwtType::Other(s.to_string()),
        })
    }
}

impl_serde_via_str!(JwtType);

/// The `typ` (and `cty`) a kind of claims is signed with, and which `typ` it accepts.
pub trait HasJwtType {
    const TYP: JwtType;
    const CTY: Option<&'static str> = None;

    /// The `typ` this value is signed with; differs from [`Self::TYP`] when it depends on the
    /// data model version.
    fn jwt_type(&self) -> JwtType {
        Self::TYP
    }

    /// The `cty` this value is signed with.
    fn content_type(&self) -> Option<&'static str> {
        Self::CTY
    }

    /// Strict by default: only [`Self::TYP`].
    fn accepts_typ(typ: Option<&JwtType>) -> bool {
        typ == Some(&Self::TYP)
    }
}
