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

#[derive(Debug, Serialize, Deserialize)]
pub struct LegalPersonCredentialSubject {
    pub id: String,
    #[serde(rename = "gx:registrationNumber")]
    pub gx_registration_number: RegistrationNumberRef,
    #[serde(rename = "gx:legalAddress")]
    pub gx_legal_address: Address,
    #[serde(rename = "gx:headquartersAddress")]
    pub gx_headquarters_address: Address,
    #[serde(rename = "name")]
    pub schema_name: String,
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub schema_description: Option<String>,
}

/// Link to the subject of the registration number credential (`gx:RegistrationNumber`).
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationNumberRef {
    pub id: String,
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
    #[serde(
        rename = "vcard:street-address",
        skip_serializing_if = "Option::is_none"
    )]
    pub street_address: Option<String>,
}
