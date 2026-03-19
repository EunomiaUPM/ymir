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

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::types::present::{Missing, Present};

#[derive(Debug, Serialize, Deserialize)]
pub struct Eori {
    pub id: String,
    // The Economic Operators Registration and Identification number (EORI).
    #[serde(rename = "gx:eori")]
    pub eori: String,
    // The country where the EORI is registered.
    #[serde(rename = "gx:country", skip_serializing_if = "Option::is_none")]
    pub country: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EoriBuilder<T> {
    id: Option<String>,
    #[serde(rename = "gx:eori")]
    eori: String,
    #[serde(rename = "gx:country", skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    #[serde(skip)]
    _marker: PhantomData<T>
}

impl EoriBuilder<Missing> {
    pub fn new(eori: String) -> Self {
        Self { id: None, eori, country: None, _marker: PhantomData }
    }
}

impl<T> EoriBuilder<T> {
    pub fn id(self, id: String) -> EoriBuilder<Present> {
        EoriBuilder { id: Some(id), eori: self.eori, country: self.country, _marker: PhantomData }
    }

    pub fn country(mut self, country: String) -> Self {
        self.country = Some(country);
        self
    }
}

impl EoriBuilder<Present> {
    pub fn build(self) -> Eori {
        Eori {
            id: self.id.expect("Builder invariant violated: id missing"),
            eori: self.eori,
            country: self.country
        }
    }
}
