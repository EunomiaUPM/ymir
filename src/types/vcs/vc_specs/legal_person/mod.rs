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
    #[serde(rename = "schema:description", skip_serializing_if = "Option::is_none")]
    pub schema_description: Option<String>,
}

/// Nested registration number embedded inside a `gx:LegalPerson` credential
/// subject. Per `gx:RegistrationNumberShape` (sh:closed false) the expected
/// pattern is to embed the subtype properties (gx:vatID, schema:leiCode, ...)
/// directly. The `registrationNumberType`/`registrationNumberValue` pair below
/// is a legacy shim we keep for backwards compat; consider replacing with the
/// proper embedded subtype variants.
#[derive(Debug, Serialize, Deserialize)]
pub struct TypedRegistrationNumber {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "gx:registrationNumberType")]
    pub gx_registration_number_type: String,
    #[serde(rename = "gx:registrationNumberValue")]
    pub gx_registration_number_value: String,
}

/// Address per `gx:AddressShape` (sh:closed true).
///
/// Most string properties live in the vCard namespace (`vcard:postal-code`,
/// `vcard:locality`, `vcard:street-address`). Only `countryCode` and
/// `countryName` are in the `gx:` namespace.
#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "@type")]
    pub r#type: String,
    /// ISO 3166-1 alpha-2/alpha-3/numeric. REQUIRED per spec.
    #[serde(rename = "gx:countryCode")]
    pub country_code: String,
    /// Plain-text country name. OPTIONAL per spec.
    #[serde(rename = "gx:countryName", skip_serializing_if = "Option::is_none")]
    pub country_name: Option<String>,
    /// vCard locality (city/town). OPTIONAL.
    #[serde(rename = "vcard:locality", skip_serializing_if = "Option::is_none")]
    pub locality: Option<String>,
    /// vCard postal code. OPTIONAL.
    #[serde(rename = "vcard:postal-code", skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// vCard street address. OPTIONAL.
    #[serde(rename = "vcard:street-address", skip_serializing_if = "Option::is_none")]
    pub street_address: Option<String>,
}

impl LegalPersonCredentialSubject {
    pub fn new4gaia(
        kid: &str,
        vc_type: &VcType,
        code: impl Into<String>,
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
                    format!(
                        "Unable to issue a LegalPerson vc while requesting {}",
                        vc_type
                    ),
                    None,
                ));
            }
        };

        Ok(LegalPersonCredentialSubject {
            id: kid.to_string(),
            gx_registration_number: TypedRegistrationNumber {
                id: None,
                gx_registration_number_type: vc_type.to_string(),
                gx_registration_number_value: code.into(),
            },
            gx_legal_address: Address {
                id: None,
                r#type: "gx:Address".to_string(),
                country_code: "ES".to_string(),
                country_name: None,
                locality: Some("Madrid".to_string()),
                postal_code: Some("28040".to_string()),
                street_address: Some(
                    "Av. Complutense, 30, Moncloa - Aravaca, 28040 Madrid".to_string(),
                ),
            },
            gx_headquarters_address: Address {
                id: None,
                r#type: "gx:Address".to_string(),
                country_code: "ES".to_string(),
                country_name: None,
                locality: Some("Madrid".to_string()),
                postal_code: Some("28040".to_string()),
                street_address: Some(
                    "Av. Complutense, 30, Moncloa - Aravaca, 28040 Madrid".to_string(),
                ),
            },
            schema_name: "UPM to the sky".to_string(),
            schema_description: None,
        })
    }
}
