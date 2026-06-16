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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cryptosuite {
    // EdDSA — vc-di-eddsa
    EddsaRdfc2022,
    EddsaJcs2022,

    // ECDSA — vc-di-ecdsa
    EcdsaRdfc2019,
    EcdsaJcs2019,

    // BBS — vc-di-bbs (selective disclosure)
    BbsRdfc2023,

    // Legacy
    RsaSignature2018,

    // Catch-all
    Other(String),
}

impl Display for Cryptosuite {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Cryptosuite::EddsaRdfc2022 => "eddsa-rdfc-2022",
            Cryptosuite::EddsaJcs2022 => "eddsa-jcs-2022",
            Cryptosuite::EcdsaRdfc2019 => "ecdsa-rdfc-2019",
            Cryptosuite::EcdsaJcs2019 => "ecdsa-jcs-2019",
            Cryptosuite::BbsRdfc2023 => "bbs-rdfc-2023",
            Cryptosuite::RsaSignature2018 => "RsaSignature2018",
            Cryptosuite::Other(s) => s.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Cryptosuite {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "eddsa-rdfc-2022" => Cryptosuite::EddsaRdfc2022,
            "eddsa-jcs-2022" => Cryptosuite::EddsaJcs2022,
            "ecdsa-rdfc-2019" => Cryptosuite::EcdsaRdfc2019,
            "ecdsa-jcs-2019" => Cryptosuite::EcdsaJcs2019,
            "bbs-rdfc-2023" => Cryptosuite::BbsRdfc2023,
            "RsaSignature2018" => Cryptosuite::RsaSignature2018,
            _ => Cryptosuite::Other(s.to_string()),
        })
    }
}

impl_serde_via_str!(Cryptosuite);
