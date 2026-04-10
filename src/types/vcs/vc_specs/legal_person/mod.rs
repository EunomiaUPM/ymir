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

use serde::{Deserialize, Serialize};

use crate::errors::{Errors, Outcome};
use crate::types::vcs::VcType;

#[derive(Debug, Serialize, Deserialize)]
pub struct LegalPersonCredentialSubject {
    pub id: String,
    #[serde(rename = "gx:registrationNumber")]
    pub gx_registration_number: TypedRegistrationNumber,
    #[serde(rename = "gx:legalAddress")]
    pub gx_legal_address: Address,
    #[serde(rename = "gx:headquartersAddress")]
    pub gx_headquarters_address: Address,
    #[serde(rename = "schema:name")]
    pub schema_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_description: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypedRegistrationNumber {
    pub id: String,
    #[serde(rename = "gx:registrationNumberType")]
    pub gx_registration_number_type: String,
    #[serde(rename = "gx:registrationNumberValue")]
    pub gx_registration_number_value: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub r#type: String,
    pub gx_country: String,
    pub gx_locality: String,
    #[serde(rename = "gx:postalCode")]
    pub gx_postal_code: String,
    #[serde(rename = "gx:streetAddress")]
    pub gx_street_address: String
}

impl LegalPersonCredentialSubject {
    pub fn new4gaia(
        kid: &str,
        vc_type: &VcType,
        code: impl Into<String>
    ) -> Outcome<LegalPersonCredentialSubject> {
        match vc_type {
            VcType::Eori
            | VcType::Euid
            | VcType::LeiCode
            | VcType::LocalRegistrationNumber
            | VcType::TaxId
            | VcType::VatId => vc_type,
            vc_type => {
                return Err(Errors::crazy(
                    format!("Unable to issue a LegalPerson vc while requesting {}", vc_type),
                    None
                ));
            }
        };

        Ok(LegalPersonCredentialSubject {
            id: kid.to_string(),
            gx_registration_number: TypedRegistrationNumber {
                id: kid.to_string(),
                gx_registration_number_type: vc_type.to_string(),
                gx_registration_number_value: code.into()
            },
            gx_legal_address: Address {
                id: None,
                r#type: "gx:Address".to_string(),
                gx_country: "ES".to_string(),
                gx_locality: "Madrid".to_string(),
                gx_postal_code: "28035".to_string(),
                gx_street_address: "Av. Complutense, 30, Moncloa - Aravaca, 28040 Madrid"
                    .to_string()
            },
            gx_headquarters_address: Address {
                id: None,
                r#type: "gx:Address".to_string(),
                gx_country: "ES".to_string(),
                gx_locality: "Madrid".to_string(),
                gx_postal_code: "28035".to_string(),
                gx_street_address: "Av. Complutense, 30, Moncloa - Aravaca, 28040 Madrid"
                    .to_string()
            },
            schema_name: "UPM to the sky".to_string(),
            schema_description: None
        })
    }
}
