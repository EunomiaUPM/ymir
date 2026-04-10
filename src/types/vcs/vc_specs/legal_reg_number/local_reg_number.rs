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
pub struct LocalRegistrationNumber {
    pub id: String,
    // The state issued company number.
    #[serde(rename = "gx:local")]
    pub local: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalRegistrationNumberBuilder<T> {
    id: Option<String>,

    // The state issued company number.
    #[serde(rename = "gx:local")]
    local: String,

    #[serde(skip)]
    _marker: PhantomData<T>
}

impl LocalRegistrationNumberBuilder<Missing> {
    pub fn new(local: String) -> Self { Self { id: None, local, _marker: PhantomData } }
}

impl<T> LocalRegistrationNumberBuilder<T> {
    pub fn id(self, id: String) -> LocalRegistrationNumberBuilder<Present> {
        LocalRegistrationNumberBuilder { id: Some(id), local: self.local, _marker: PhantomData }
    }
}

impl LocalRegistrationNumberBuilder<Present> {
    pub fn build(self) -> LocalRegistrationNumber {
        LocalRegistrationNumber {
            id: self.id.expect("Builder invariant violated: id missing"),
            local: self.local
        }
    }
}
