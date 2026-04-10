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

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::types::present::{Missing, Present};

#[derive(Debug, Serialize, Deserialize)]
pub struct Euid {
    pub id: String,
    // The European Unique Identifier (EUID).
    #[serde(rename = "gx:euid")]
    pub euid: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EuidBuilder<T> {
    id: Option<String>,

    // The European Unique Identifier (EUID).
    #[serde(rename = "gx:euid")]
    euid: String,

    #[serde(skip)]
    _marker: PhantomData<T>
}

impl EuidBuilder<Missing> {
    pub fn new(euid: String) -> Self { Self { id: None, euid, _marker: PhantomData } }
}

impl<T> EuidBuilder<T> {
    pub fn id(self, id: String) -> EuidBuilder<Present> {
        EuidBuilder { id: Some(id), euid: self.euid, _marker: PhantomData }
    }
}

impl EuidBuilder<Present> {
    pub fn build(self) -> Euid {
        Euid { id: self.id.expect("Builder invariant violated: id missing"), euid: self.euid }
    }
}
