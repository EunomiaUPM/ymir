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

use super::KeyRef;
use crate::capabilities::Did;
use crate::types::dids::DidDocument;

#[derive(Debug, Clone)]
pub struct Identity {
    did: Did,
    did_doc: DidDocument,
    keys_ref: KeyRef,
}

impl Identity {
    pub fn new(did: Did, did_doc: DidDocument, keys_ref: KeyRef) -> Self {
        Self {
            did,
            did_doc,
            keys_ref,
        }
    }
    pub fn did(&self) -> &Did {
        &self.did
    }
    pub fn did_doc(&self) -> &DidDocument {
        &self.did_doc
    }
    pub fn key_ref(&self) -> &KeyRef {
        &self.keys_ref
    }
}
