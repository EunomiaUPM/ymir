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

use super::{DidService, VerificationMethod};
use crate::capabilities::Did;
use crate::errors::Outcome;
use crate::types::keys::PrivateKey;
use crate::utils::StringOrArr;
use sea_orm::{FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: StringOrArr,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<StringOrArr>, // TODO
    #[serde(rename = "alsoKnownAs", skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<StringOrArr>, // TODO
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Vec<DidService>>,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<StringOrArr>, // TODO
    #[serde(rename = "assertionMethod", skip_serializing_if = "Option::is_none")]
    pub assertion_method: Option<StringOrArr>, // TODO
    #[serde(rename = "keyAgreement", skip_serializing_if = "Option::is_none")]
    pub key_agreement: Option<StringOrArr>, // TODO
    #[serde(
        rename = "capabilityInvocation",
        skip_serializing_if = "Option::is_none"
    )]
    pub capability_invocation: Option<StringOrArr>, // TODO
    #[serde(
        rename = "capabilityDelegation",
        skip_serializing_if = "Option::is_none"
    )]
    pub capability_delegation: Option<StringOrArr>, // TODO
}

impl DidDocument {
    pub fn base(did: &Did, key_with_frag: Vec<(PrivateKey, String)>) -> DidDocument {
        let vms: Vec<VerificationMethod> = key_with_frag
            .iter()
            .map(|(key, vm_frag)| VerificationMethod::new(did, &key, &vm_frag))
            .collect();

        DidDocument {
            context: StringOrArr::Arr(vec!["https://www.w3.org/ns/did/v1.1".to_string()]),
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

    pub fn get_did(&self) -> Outcome<Did> {
        Did::parse(&self.id)
    }

    pub fn add_services(mut self, services: Vec<DidService>) -> Self {
        self.service = Some(services);
        self
    }

    pub fn add_key(&mut self, key: &PrivateKey, vm_frag: Option<&str>) {
        let did = Did::parse(&self.id).unwrap(); // THE CREATION MAKES PANIC IMPOSSIBLE
        let len = self.verification_method.len().to_string();
        let key_frag = vm_frag.unwrap_or(&len).to_string();
        self.verification_method
            .push(VerificationMethod::new(&did, &key, &key_frag));
    }

    pub fn delete_key(&mut self, vm_frag: &str) {
        let did = Did::parse(&self.id).unwrap(); // THE CREATION MAKES PANIC IMPOSSIBLE
        let vm_id = format!("{}#{}", did.id(), vm_frag);
        self.verification_method
            .retain(|vm| vm.id != vm_id);
    }
}
