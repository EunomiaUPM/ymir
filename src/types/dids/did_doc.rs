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

use rayon::prelude::*;
use super::{DidService, VerificationMethod};
use crate::capabilities::Did;
use crate::types::keys::Key;
use serde::{Deserialize, Serialize};
use crate::utils::HasId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: String,
    pub id: String,
    pub controller: Option<String>, // TODO
    #[serde(rename = "alsoKnownAs")]
    pub also_known_as: Option<String>, // TODO
    pub service: Option<Vec<DidService>>,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Option<String>, // TODO
    #[serde(rename = "assertionMethod")]
    pub assertion_method: Option<String>, // TODO
    #[serde(rename = "keyAgreement")]
    pub key_agreement: Option<String>, // TODO
    #[serde(rename = "capabilityInvocation")]
    pub capability_invocation: Option<String>, // TODO
    #[serde(rename = "capabilityDelegation")]
    pub capability_delegation: Option<String>, // TODO
}

impl DidDocument {
    pub fn base(did: &Did, key: &[Key]) -> DidDocument {
        let vms: Vec<VerificationMethod> = key.par_iter().map(
            |key| VerificationMethod::new(did, key)
        ).collect();

        DidDocument {
            context: "https://www.w3.org/ns/did/v1.1".to_string(),
            id: did.id().to_string(),
            controller: None,
            also_known_as: None,
            service: None,
            verification_method: vms,
            authentication: None,
            assertion_method: None,
            key_agreement: None,
            capability_invocation: None,
            capability_delegation: None,
        }
    }
    pub fn add_services(mut self, services: Vec<DidService>) -> Self {
        self.service = Some(services);
        self
    }
}
