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
pub struct LeiCode {
    pub id: String,
    // Unique LEI number.
    #[serde(rename = "gx:leiCode")]
    pub lei_code: String,
    // The country subdivision (state/region) where the LEI number is registered.
    #[serde(rename = "gx:city", skip_serializing_if = "Option::is_none")]
    pub subdivision_country_code: Option<String>,
    // The country where the LEI number is registered.
    #[serde(rename = "gx:country")]
    pub country_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeiCodeBuilder<T> {
    id: Option<String>,

    // Unique LEI number.
    #[serde(rename = "gx:leiCode")]
    lei_code: String,

    // The country subdivision (state/region) where the LEI number is registered.
    #[serde(rename = "gx:city", skip_serializing_if = "Option::is_none")]
    subdivision_country_code: Option<String>,

    // The country where the LEI number is registered.
    #[serde(rename = "gx:country")]
    country_code: String,

    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl LeiCodeBuilder<Missing> {
    pub fn new(lei_code: String, country_code: String) -> Self {
        Self {
            id: None,
            lei_code,
            subdivision_country_code: None,
            country_code,
            _marker: PhantomData,
        }
    }
}

impl<T> LeiCodeBuilder<T> {
    pub fn id(self, id: String) -> LeiCodeBuilder<Present> {
        LeiCodeBuilder {
            id: Some(id),
            lei_code: self.lei_code,
            subdivision_country_code: self.subdivision_country_code,
            country_code: self.country_code,
            _marker: PhantomData,
        }
    }

    pub fn subdivision_country_code(mut self, code: String) -> Self {
        self.subdivision_country_code = Some(code);
        self
    }
}

impl LeiCodeBuilder<Present> {
    pub fn build(self) -> LeiCode {
        LeiCode {
            id: self.id.expect("Builder invariant violated: id missing"),
            lei_code: self.lei_code,
            subdivision_country_code: self.subdivision_country_code,
            country_code: self.country_code,
        }
    }
}
