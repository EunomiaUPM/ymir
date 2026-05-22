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
//! Verifiable Credential types following the W3C VC Data Model 2.0.
//!
//! The central type is [`VcDocument`]. The remaining types model optional
//! fields of a VC:
//!
//! - [`TermsOfUse`] — links the credential to a governing VDS via
//!   `digestSRI`. This is the integration point with the VDS protocol.
//! - [`VCSchema`], [`VCStatus`], [`VCEvidence`], [`VCRefreshService`] —
//!   standard W3C VC fields.
//! - [`DSParticipantCredSub`] — the `credentialSubject` shape for a
//!   Data Space Participant credential (specific to the paper's example).
//! - [`VDSCredentialType`] — the schema entry declared inside a VDS's
//!   `credentialTypes` list.

mod base_issuer;
mod evidence;
mod refresh_service;
mod schema;
mod status;
mod terms_of_use;
mod vc_doc;
mod vc_builder;

pub use evidence::VCEvidence;
pub use refresh_service::VCRefreshService;
pub use schema::*;
pub use status::VCStatus;
pub use terms_of_use::TermsOfUse;
pub use vc_doc::VcDocument;
pub use base_issuer::BaseIssuer;