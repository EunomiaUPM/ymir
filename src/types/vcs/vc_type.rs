/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::{BadFormat, Errors};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VcType {
    Eori,
    Euid,
    LeiCode,
    LocalRegistrationNumber,
    TaxId,
    VatId,
    DataspaceParticipant,
    LegalPerson,
    TermsAndConditions
}

impl FromStr for VcType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gx:Eori" => Ok(VcType::Eori),
            "gx:Euid" => Ok(VcType::Euid),
            "gx:LeiCode" => Ok(VcType::LeiCode),
            "gx:LocalRegistrationNumber" => Ok(VcType::LocalRegistrationNumber),
            "gx:TaxId" => Ok(VcType::TaxId),
            "gx:VatId" => Ok(VcType::VatId),
            "DataspaceParticipant" => Ok(VcType::DataspaceParticipant),
            "LegalPerson" => Ok(VcType::LegalPerson),
            "TermsAndConditions" => Ok(VcType::TermsAndConditions),
            format => Err(Errors::parse(format!("Unknown credential format: {}", format), None))
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
            VcType::DataspaceParticipant => "DataspaceParticipant",
            VcType::LegalPerson => "LegalPerson",
            VcType::TermsAndConditions => "TermsAndConditions"
        };

        write!(f, "{s}")
    }
}

impl VcType {
    pub fn to_conf(&self) -> String {
        match self {
            VcType::Eori => "gx_Eori_jwt_vc_json",
            VcType::Euid => "gx_Euid_jwt_vc_json",
            VcType::LeiCode => "gx_LeiCode_jwt_vc_json",
            VcType::LocalRegistrationNumber => "gx_LocalRegistrationNumber_jwt_vc_json",
            VcType::TaxId => "gx_TaxId_jwt_vc_json",
            VcType::VatId => "gx_VatId_jwt_vc_json",
            VcType::DataspaceParticipant => "DataspaceParticipant_jwt_vc_json",
            VcType::LegalPerson => "LegalPerson_jwt_vc_json",
            VcType::TermsAndConditions => "TermsAndConditions_jwt_vc_json"
        }
        .to_string()
    }

    pub fn from_conf(s: &str) -> Result<Self, Errors> {
        match s {
            "gx_Eori_jwt_vc_json" => Ok(VcType::Eori),
            "gx_Euid_jwt_vc_json" => Ok(VcType::Euid),
            "gx_LeiCode_jwt_vc_json" => Ok(VcType::LeiCode),
            "gx_LocalRegistrationNumber_jwt_vc_json" => Ok(VcType::LocalRegistrationNumber),
            "gx_TaxId_jwt_vc_json" => Ok(VcType::TaxId),
            "gx_VatId_jwt_vc_json" => Ok(VcType::VatId),
            "DataspaceParticipant_jwt_vc_json" => Ok(VcType::DataspaceParticipant),
            "LegalPerson_jwt_vc_json" => Ok(VcType::LegalPerson),
            "TermsAndConditions_jwt_vc_json" => Ok(VcType::TermsAndConditions),
            _ => Err(Errors::format(
                BadFormat::Received,
                format!("Unknown credential configuration: {}", s),
                None
            ))
        }
    }

    pub fn variants() -> &'static [VcType] {
        &[
            VcType::Eori,
            VcType::Euid,
            VcType::LeiCode,
            VcType::LocalRegistrationNumber,
            VcType::TaxId,
            VcType::VatId,
            VcType::DataspaceParticipant,
            VcType::LegalPerson,
            VcType::TermsAndConditions
        ]
    }
}
