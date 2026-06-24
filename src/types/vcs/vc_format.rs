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

use crate::impl_serde_via_str;
use sea_orm::FromJsonQueryResult;
use std::convert::Infallible;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, FromJsonQueryResult)]
pub enum VcFormat {
    JwtVcJson,
    JwtVcJsonLd,
    LdpVc,
    SdJwtVc,
    MsoMdoc,
    Other(String),
}

impl Display for VcFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            VcFormat::JwtVcJson => "jwt_vc_json",
            VcFormat::JwtVcJsonLd => "jwt_vc_json-ld",
            VcFormat::LdpVc => "ldp_vc",
            VcFormat::SdJwtVc => "vc+sd-jwt",
            VcFormat::MsoMdoc => "mso_mdoc",
            VcFormat::Other(s) => s.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for VcFormat {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "jwt_vc_json" => VcFormat::JwtVcJson,
            "jwt_vc_json-ld" => VcFormat::JwtVcJsonLd,
            "ldp_vc" => VcFormat::LdpVc,
            "vc+sd-jwt" => VcFormat::SdJwtVc,
            "mso_mdoc" => VcFormat::MsoMdoc,
            other => VcFormat::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(VcFormat);

impl VcFormat {
    pub fn supported() -> &'static [VcFormat] {
        &[VcFormat::JwtVcJson]
    }
    pub fn is_supported(&self) -> bool {
        Self::supported().contains(self)
    }
}
