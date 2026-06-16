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
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromJsonQueryResult)]
pub enum VcType {
    Eori,
    Euid,
    LeiCode,
    LocalRegistrationNumber,
    TaxId,
    VatId,
    DataspaceParticipant,
    LegalPerson,
    TermsAndConditions,
    GxLabel,
    Other(String),
}

impl FromStr for VcType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "gx:Eori" => Ok(VcType::Eori),
            "gx:Euid" => Ok(VcType::Euid),
            "gx:LeiCode" => Ok(VcType::LeiCode),
            "gx:LocalRegistrationNumber" => Ok(VcType::LocalRegistrationNumber),
            "gx:TaxId" => Ok(VcType::TaxId),
            "gx:VatId" => Ok(VcType::VatId),
            "DataSpaceParticipant" => Ok(VcType::DataspaceParticipant),
            "gx:LegalPerson" => Ok(VcType::LegalPerson),
            "gx:TermsAndConditions" => Ok(VcType::TermsAndConditions),
            "gx:LabelCredential" => Ok(VcType::GxLabel),
            other => Ok(VcType::Other(other.to_string())),
        }
    }
}

impl Display for VcType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            VcType::Eori => "gx:Eori",
            VcType::Euid => "gx:Euid",
            VcType::LeiCode => "gx:LeiCode",
            VcType::LocalRegistrationNumber => "gx:LocalRegistrationNumber",
            VcType::TaxId => "gx:TaxId",
            VcType::VatId => "gx:VatId",
            VcType::DataspaceParticipant => "DataSpaceParticipant",
            VcType::LegalPerson => "gx:LegalPerson",
            VcType::TermsAndConditions => "gx:TermsAndConditions",
            VcType::GxLabel => "gx:LabelCredential",
            VcType::Other(other) => { other }
        };

        write!(f, "{s}")
    }
}

impl VcType {
    pub fn supported() -> Vec<VcType> {
        vec![
            VcType::Eori,
            VcType::Euid,
            VcType::LeiCode,
            VcType::LocalRegistrationNumber,
            VcType::TaxId,
            VcType::VatId,
            VcType::DataspaceParticipant,
            VcType::LegalPerson,
            VcType::TermsAndConditions,
        ]
    }
    // pub fn to_gaia_weird(&self) -> Outcome<&str> {
    //     match self {
    //         VcType::Eori => Ok("gx:eori"),
    //         VcType::Euid => Ok("gx:euid"),
    //         VcType::LeiCode => Ok("gx:leiCode"),
    //         VcType::LocalRegistrationNumber => Ok("gx:local"),
    //         VcType::TaxId => Ok("gx:taxID"),
    //         VcType::VatId => Ok("gx:vatID"),
    //         VcType::LegalPerson => Ok("gx:legalPerson"),
    //         VcType::TermsAndConditions => Ok("gx:termsAndConditions"),
    //         vc_type => Err(Errors::format(
    //             BadFormat::Received,
    //             format!("Cannot implement this function with vc_type {}", vc_type),
    //             None,
    //         )),
    //     }
    // }
}
