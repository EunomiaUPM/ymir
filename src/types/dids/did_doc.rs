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

use super::Did;
use super::{DidService, VerificationMethod};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::crypto::ProofPurpose;
use crate::types::dids::kid::Kid;
use crate::types::keys::{PrivateKey, PublicKey};
use crate::utils::OneOrMany;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: OneOrMany<String>,
    pub id: Did,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<OneOrMany<String>>, // TODO
    #[serde(rename = "alsoKnownAs", skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<OneOrMany<String>>, // TODO
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Vec<DidService>>,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<OneOrMany<String>>, // TODO
    #[serde(rename = "assertionMethod", skip_serializing_if = "Option::is_none")]
    pub assertion_method: Option<OneOrMany<String>>, // TODO
    #[serde(rename = "keyAgreement", skip_serializing_if = "Option::is_none")]
    pub key_agreement: Option<OneOrMany<String>>, // TODO
    #[serde(
        rename = "capabilityInvocation",
        skip_serializing_if = "Option::is_none"
    )]
    pub capability_invocation: Option<OneOrMany<String>>, // TODO
    #[serde(
        rename = "capabilityDelegation",
        skip_serializing_if = "Option::is_none"
    )]
    pub capability_delegation: Option<OneOrMany<String>>, // TODO
}

impl DidDocument {
    pub fn from_vms(did_id: &Did, vms: Vec<VerificationMethod>) -> DidDocument {
        let vm_ids = OneOrMany::Many(vms.iter().map(|vm| vm.id.clone()).collect());

        DidDocument {
            context: OneOrMany::Many(vec!["https://www.w3.org/ns/did/v1.1".to_string()]),
            id: did_id.clone(),
            controller: Some(OneOrMany::One(did_id.to_string())),
            also_known_as: None,
            service: None,
            authentication: Some(vm_ids.clone()),
            assertion_method: Some(vm_ids.clone()),
            key_agreement: None,
            capability_invocation: Some(vm_ids.clone()),
            capability_delegation: Some(vm_ids),
            verification_method: vms,
        }
    }
    pub fn base(did: &Did, key_with_frag: Vec<(PrivateKey, String)>) -> DidDocument {
        let vms = key_with_frag
            .iter()
            .map(|(key, vm_frag)| VerificationMethod::new(did, key, vm_frag))
            .collect();
        Self::from_vms(did, vms)
    }

    pub fn get_did(&self) -> &Did {
        &self.id
    }

    /// Verification methods the document authorises for a given proof purpose.
    pub fn relationship(&self, purpose: &ProofPurpose) -> Option<&OneOrMany<String>> {
        match purpose {
            ProofPurpose::Authentication => self.authentication.as_ref(),
            ProofPurpose::AssertionMethod => self.assertion_method.as_ref(),
            ProofPurpose::KeyAgreement => self.key_agreement.as_ref(),
            ProofPurpose::CapabilityInvocation => self.capability_invocation.as_ref(),
            ProofPurpose::CapabilityDelegation => self.capability_delegation.as_ref(),
            ProofPurpose::Other(_) => None,
        }
    }

    pub fn add_services(&mut self, services: Vec<DidService>) {
        self.service = Some(services);
    }

    /// Adds a key to the document as a general-purpose signing key: it is referenced
    /// from every relationship a signing key is authorised for, never from
    /// `keyAgreement`, which belongs to encryption keys. Per-purpose separation
    /// (a key that may authenticate but not assert) is not modelled.
    pub fn add_key(&mut self, key: &PrivateKey, vm_frag: Option<&str>) {
        let len = self.verification_method.len().to_string();
        let frag = vm_frag.unwrap_or(&len).to_string();

        let vm = VerificationMethod::new(&self.id, key, &frag);
        let vm_id = vm.id.clone();
        self.verification_method.push(vm);

        for rel in [
            &mut self.authentication,
            &mut self.assertion_method,
            &mut self.capability_invocation,
            &mut self.capability_delegation,
        ] {
            rel.get_or_insert_with(|| OneOrMany::Many(Vec::new()))
                .push(vm_id.clone());
        }
    }

    /// Removes a key from the document and from every relationship that referenced
    /// it. Every relationship is swept, `keyAgreement` included: the document may
    /// have been produced elsewhere, and a reference left behind would point at a
    /// verification method that no longer exists.
    pub fn delete_key(&mut self, vm_frag: &str) {
        let vm_id = format!("{}#{}", self.id, vm_frag);

        self.verification_method.retain(|vm| vm.id != vm_id);

        for rel in [
            &mut self.authentication,
            &mut self.assertion_method,
            &mut self.key_agreement,
            &mut self.capability_invocation,
            &mut self.capability_delegation,
        ] {
            if let Some(ids) = rel {
                ids.retain(|id| id != &vm_id);
                if ids.is_empty() {
                    *rel = None;
                }
            }
        }
    }

    /// Returns the key referenced by `kid`, but only if this document authorises
    /// that verification method for `purpose`. Purely local.
    pub fn resolve_key(&self, kid: &Kid, purpose: &ProofPurpose) -> Outcome<PublicKey> {
        self.validate_key(kid, purpose)?;
        self.extract_key(kid)
    }

    /// Whether the vm is listed under the relationship for `purpose`.
    /// CID v1.0: if it is not associated with the relationship, an error MUST be raised.
    fn validate_key(&self, kid: &Kid, purpose: &ProofPurpose) -> Outcome<()> {
        let target = kid.to_string();
        let authorised = self
            .relationship(purpose)
            .is_some_and(|ids| ids.iter().any(|id| self.id_matches(id, &target)));

        if !authorised {
            return Err(Errors::forbidden(
                format!("'{target}' is not authorised for {purpose}"),
                None,
            ));
        }
        Ok(())
    }

    /// Finds the verification method by (normalised) id and parses its key.
    fn extract_key(&self, kid: &Kid) -> Outcome<PublicKey> {
        let target = kid.to_string();
        let vm = self
            .verification_method
            .iter()
            .find(|vm| self.id_matches(&vm.id, &target))
            .ok_or_else(|| {
                Errors::format(
                    BadFormat::Received,
                    format!("Verification method '{target}' not found"),
                    None,
                )
            })?;
        PublicKey::parse_from_vm(vm)
    }

    /// Matches a listed id (absolute or relative `#frag`) against an absolute target.
    fn id_matches(&self, listed: &str, target: &str) -> bool {
        match listed.strip_prefix('#') {
            Some(frag) => format!("{}#{}", self.id, frag) == target,
            None => listed == target,
        }
    }
}
