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

use super::{Did, DidType};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::impl_serde_via_str;
use crate::types::keys::PublicKey;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Key Identifier (KID) structural parser and cryptographic key resolver.
///
/// Dissects standard compound identification URIs containing a foundational
/// Decentralized Identifier (DID) and its corresponding cryptographic key verification fragment identifier.
#[derive(Clone, Debug, PartialEq)]
pub struct Kid {
    did: Did,
    frag_id: String,
}

impl Kid {
    pub fn new(did: Did, frag_id: impl Into<String>) -> Self {
        Self {
            did,
            frag_id: frag_id.into(),
        }
    }

    // ===== PARSING & CONSTRUCTION ================================================================

    /// Parses a raw string slice identifier representation into a validated concrete [`Kid`] instance.
    ///
    /// # Errors
    /// Returns an [`Errors::FormatError`] if the incoming payload string fails to present a trailing
    /// URI fragment separator character (`#`) or if the fragment itself evaluation yields empty strings.
    pub fn parse(kid: &str) -> Outcome<Kid> {
        let (did, frag_id) = kid.split_once('#').ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                format!("Kid '{kid}' must include a fragment"),
                None,
            )
        })?;

        if frag_id.is_empty() {
            return Err(Errors::format(
                BadFormat::Received,
                format!("Kid '{kid}' has an empty fragment"),
                None,
            ));
        }

        Ok(Kid {
            frag_id: frag_id.to_string(),
            did: Did::parse(did)?,
        })
    }

    // ===== PROPERTY ACCESSORS ====================================================================

    /// Resolves the baseline taxonomy scheme classification type governing the underlying inner DID.
    pub fn r#type(&self) -> DidType {
        self.did.r#type()
    }

    /// Yields a reference to the parsed polymorphic decentralized identifier instance.
    pub fn did(&self) -> &Did {
        &self.did
    }

    // ===== RESOLUTION WORKFLOWS ==================================================================

    /// Triggers the downstream DID Document resolution pipeline to extract the target matching [`PublicKey`],
    /// without checking that it is authorised for any particular purpose. Prefer
    /// [`super::DidDocument::resolve_key`] when a proof purpose applies.
    ///
    /// # Errors
    /// Returns an [`Errors::FormatError`] if the designated fragment identifier fails to match
    /// any verification methods listed inside the recovered canonical structural data document.
    pub async fn get_key(&self) -> Outcome<PublicKey> {
        let did_doc = self.did.resolve().await?;
        let target = self.to_string();

        let vm = did_doc
            .verification_method
            .iter()
            .find(|vm| match vm.id.strip_prefix('#') {
                Some(frag) => format!("{}#{}", did_doc.id, frag) == target,
                None => vm.id == target,
            })
            .ok_or_else(|| {
                Errors::format(
                    BadFormat::Received,
                    format!(
                        "Verification method '{}' not found in DID Document for {}",
                        self.frag_id,
                        self.did.id()
                    ),
                    None,
                )
            })?;

        PublicKey::parse_from_vm(vm)
    }
}

impl Display for Kid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.did, self.frag_id)
    }
}

impl FromStr for Kid {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Kid::parse(s)
    }
}

impl_serde_via_str!(Kid);
