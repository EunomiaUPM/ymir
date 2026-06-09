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

use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::dids::{DidType};
use crate::types::keys::{Alg, PublicKey};
use crate::capabilities::Did;

pub struct Kid {
    frag_id: String,
    did: Did,
}


impl Kid {
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

    pub fn r#type(&self) -> DidType {
        match self.did {
            Did::Jwk(_) => DidType::Jwk,
            Did::Web(_) => DidType::Web,
        }
    }

    pub fn did(&self) -> &Did {
        &self.did
    }

    pub async fn get_key(&self, alg: &Alg) -> Outcome<PublicKey> {
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

        PublicKey::parse_from(vm, alg)
    }
}
