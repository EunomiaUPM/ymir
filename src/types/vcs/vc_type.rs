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
use std::fmt::Formatter;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::Errors;
use crate::types::vcs::vc_specs::legal_authority::LegalRegistrationNumberTypes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VcType {
    LegalRegistrationNumber(LegalRegistrationNumberTypes),
    DataspaceParticipant,
    LegalPerson,
    TermsAndConditions,
    Unknown
}

impl FromStr for VcType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LegalRegistrationNumber-tax_id" => {
                Ok(VcType::LegalRegistrationNumber(LegalRegistrationNumberTypes::TaxId))
            }
            "LegalRegistrationNumber-euid" => {
                Ok(VcType::LegalRegistrationNumber(LegalRegistrationNumberTypes::Euid))
            }
            "LegalRegistrationNumber-eori" => {
                Ok(VcType::LegalRegistrationNumber(LegalRegistrationNumberTypes::Eori))
            }
            "LegalRegistrationNumber-vat_id" => {
                Ok(VcType::LegalRegistrationNumber(LegalRegistrationNumberTypes::VatId))
            }
            "LegalRegistrationNumber-lei_code" => {
                Ok(VcType::LegalRegistrationNumber(LegalRegistrationNumberTypes::LeiCode))
            }
            "DataspaceParticipant" => Ok(VcType::DataspaceParticipant),
            "LegalPerson" => Ok(VcType::LegalPerson),
            "TermsAndConditions" => Ok(VcType::TermsAndConditions),
            format => Err(Errors::parse(format!("Unknown credential format: {}", format), None))
        }
    }
}

impl fmt::Display for VcType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            VcType::LegalRegistrationNumber(data) => match data {
                LegalRegistrationNumberTypes::TaxId => "LegalRegistrationNumber-tax_id".to_string(),
                LegalRegistrationNumberTypes::Euid => "LegalRegistrationNumber-euid".to_string(),
                LegalRegistrationNumberTypes::Eori => "LegalRegistrationNumber-eori".to_string(),
                LegalRegistrationNumberTypes::VatId => "LegalRegistrationNumber-vat_id".to_string(),
                LegalRegistrationNumberTypes::LeiCode => {
                    "LegalRegistrationNumber-lei_code".to_string()
                }
            },
            VcType::DataspaceParticipant => "DataspaceParticipant".to_string(),
            VcType::LegalPerson => "LegalPerson".to_string(),
            VcType::TermsAndConditions => "TermsAndConditions".to_string(),
            _ => "Unknown".to_string()
        };

        write!(f, "{s}")
    }
}

impl VcType {
    pub fn to_conf(&self) -> String {
        match self {
            VcType::LegalRegistrationNumber(_) => "LegalRegistrationNumber_jwt_vc_json".to_string(),
            VcType::DataspaceParticipant => "DataspaceParticipant_vc_json".to_string(),
            VcType::LegalPerson => "LegalPerson_jwt_vc_json".to_string(),
            VcType::TermsAndConditions => "TermsAndConditions_jwt_vc_json".to_string(),
            _ => "Unknown".to_string()
        }
    }

    pub fn variants() -> &'static [VcType] {
        &[
            VcType::LegalRegistrationNumber(LegalRegistrationNumberTypes::TaxId),
            VcType::DataspaceParticipant,
            VcType::LegalPerson,
            VcType::TermsAndConditions
        ]
    }
    pub fn name(&self) -> String {
        match self {
            VcType::LegalRegistrationNumber(_) => "LegalRegistrationNumber".to_string(),
            VcType::DataspaceParticipant => "DataspaceParticipant".to_string(),
            VcType::LegalPerson => "LegalPerson".to_string(),
            VcType::TermsAndConditions => "TermsAndConditions".to_string(),
            VcType::Unknown => "Unknown".to_string()
        }
    }
}
