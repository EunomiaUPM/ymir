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
pub struct VatId {
    pub id: String,
    // The VAT identification number.
    #[serde(rename = "gx:vatID")]
    pub vat_id: String,
    // The country where the VAT identification number is registered.
    #[serde(rename = "gx:countryCode", skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VatIdBuilder<T> {
    id: Option<String>,

    // The VAT identification number.
    #[serde(rename = "gx:vatID")]
    vat_id: String,

    // The country where the VAT identification number is registered.
    #[serde(rename = "gx:countryCode", skip_serializing_if = "Option::is_none")]
    country_code: Option<String>,

    #[serde(skip)]
    _marker: PhantomData<T>,
}

impl VatIdBuilder<Missing> {
    pub fn new(vat_id: String) -> Self {
        Self {
            id: None,
            vat_id,
            country_code: None,
            _marker: PhantomData,
        }
    }
}

impl<T> VatIdBuilder<T> {
    pub fn id(self, id: String) -> VatIdBuilder<Present> {
        VatIdBuilder {
            id: Some(id),
            vat_id: self.vat_id,
            country_code: self.country_code,
            _marker: PhantomData,
        }
    }

    pub fn country_code(mut self, country_code: String) -> Self {
        self.country_code = Some(country_code);
        self
    }
}

impl VatIdBuilder<Present> {
    pub fn build(self) -> VatId {
        VatId {
            id: self.id.expect("Builder invariant violated: id missing"),
            vat_id: self.vat_id,
            country_code: self.country_code,
        }
    }
}
