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

use serde::{Deserialize, Serialize};
use crate::utils::HasId;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum VcIssuer {
    Did(String),
    Object(BaseIssuer),
}

impl VcIssuer {
    pub fn new(id: impl Into<String>, name: Option<impl Into<String>>) -> VcIssuer {
        match name {
            Some(name) => {
                VcIssuer::Object(BaseIssuer {
                    id: id.into(),
                    name: name.into(),
                })
            }
            None => { VcIssuer::Did(id.into()) }
        }
    }
}

impl HasId for VcIssuer {
    fn id(&self) -> &str {
        match self {
            VcIssuer::Did(did) => { did }
            VcIssuer::Object(doc) => { &doc.id }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseIssuer {
    pub id: String,
    pub name: String,
}