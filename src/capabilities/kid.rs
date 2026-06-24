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

use crate::capabilities::Did;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::dids::DidType;
use crate::types::keys::PublicKey;

/// Key Identifier (KID) structural parser and cryptographic key resolver.
///
/// Dissects standard compound identification URIs containing a foundational 
/// Decentralized Identifier (DID) and its corresponding cryptographic key verification fragment identifier.
pub struct Kid {
    frag_id: String,
    did: Did,
}

impl Kid {
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

    /// Triggers the downstream DID Document resolution pipeline to extract the target matching [`PublicKey`].
    ///
    /// # Errors
    /// Returns an [`Errors::FormatError`] if the designated fragment identifier fails to match 
    /// any verification methods listed inside the recovered canonical structural data document.
    pub async fn get_key(&self) -> Outcome<PublicKey> {
        let did_doc = self.did.resolve().await?;

        let vm = did_doc
            .verification_method
            .iter()
            .find(|vm| {
                vm.id
                    .rsplit_once('#')
                    .map(|(_, frag)| frag == self.frag_id)
                    .unwrap_or(false)
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