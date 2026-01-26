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

use crate::data::entities::req_interaction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interact4GR {
    pub start: Vec<String>,
    pub finish: Finish4Interact, // REQUIRED because DataSpace Protocol is based on redirects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Finish4Interact {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>, // REQUIRED for redirect and push methods
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_method: Option<String>,
}



impl Interact4GR {
    pub fn new(model: &req_interaction::Model) -> Interact4GR {
        Self {
            start: model.start.clone(),
            finish: Finish4Interact {
                method: model.method.clone(),
                uri: Some(model.uri.clone()),
                nonce: model.client_nonce.clone(),
                hash_method: model.hash.clone(),
            },
            hints: None,
        }
    }
}
