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

use super::HasId;
use super::KeyRef;
use crate::types::dids::{DidBuilder, DidDocument, DidService, DidType};
use crate::types::keys::PrivateKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidPlan {
    pub alias: String,
    pub r#type: DidBuilder,
    pub keys: Vec<String>,
    pub service: Option<Vec<DidService>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidModel {
    pub did_id: String,
    pub did: String,
    pub alias: String,
    pub r#default: bool,
    pub r#type: DidType,
    pub keys: Vec<KeyRef>,
    pub did_document: DidDocument,
}

impl HasId for DidModel {
    fn id(&self) -> &str {
        &self.did_id
    }
}

impl Into<DidModel> for DidPlan {
    fn into(self) -> DidModel {
        todo!()
    }
}

pub struct PreDidEntry {
    pub alias: String,
    pub r#type: DidBuilder,
    pub keys_id: Vec<String>,
    pub keys: Vec<PrivateKey>,
    pub service: Option<Vec<DidService>>,
}
