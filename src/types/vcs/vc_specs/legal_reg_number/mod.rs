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

mod eori;
mod euid;
mod leicode;
mod local_reg_number;
mod taxid;
mod vatid;
mod vc_data;

pub use eori::*;
pub use euid::*;
pub use leicode::*;
pub use local_reg_number::*;
pub use taxid::*;
pub use vatid::*;
pub use vc_data::*;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct LegalPerson {
//     // Country's registration number.
//     pub registration_number: RegistrationNumber,
//     // Full physical location of the headquarter of the organization.
//     pub headquarters_address: String, // Assuming String for simplicity, should be Address struct
//     // The full legal address of the organization.
//     pub legal_address: String, // Assuming String for simplicity, should be Address struct
//     // A human readable name of the entity.
//     pub name: Option<String>,
//     pub description: Option<String>,
//     // Gaia-X Credentials of participants that this entity is a subOrganization of.
//     pub sub_organisation_of: Vec<String>,
//     // Gaia-X Credentials of participants that this entity is a parentOrganization of.
//     pub parent_organization_of: Vec<String>,
// }
// //
// #[derive(Debug, Serialize, Deserialize)]
// pub struct TermsAndConditions {
//     // SHA-256 hash of the document.
//     pub hash: String,
//     // Resolvable link to the document.
//     pub url: String,
// }
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct CompliantCredential {
//     // Type of the compliant credential.
//     pub credential_type: String,
//     // Subresource Integrity hash of the verifiable credential.
//     pub digest_sri: String,
// }
