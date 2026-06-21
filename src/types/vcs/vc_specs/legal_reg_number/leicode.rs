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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LeiCode {
    pub id: String,
    // Unique LEI number per https://www.gleif.org (ISO 17442, 20 chars).
    #[serde(rename = "schema:leiCode")]
    pub lei_code: String,
    // The country subdivision (state/region) where the LEI number is registered.
    #[serde(rename = "gx:subdivisionCountryCode", skip_serializing_if = "Option::is_none")]
    pub subdivision_country_code: Option<String>,
    // The country where the LEI number is registered (ISO 3166).
    #[serde(rename = "gx:countryCode")]
    pub country_code: String,
}

