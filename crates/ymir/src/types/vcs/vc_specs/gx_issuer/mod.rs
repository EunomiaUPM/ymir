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

/// Hash of the Gaia-X terms and conditions a `gx:Issuer` accepts (one of the two the 2.5.0 shape allows).
pub const GAIA_X_TERMS_HASH: &str = "067dcac5efd18c1927deb1ffed3feab6d0ad044c0a9a263e6d5d8bdc43224515";

/// Subject of a `gx:Issuer`: who accepts the Gaia-X terms and conditions, and their hash.
#[derive(Debug, Serialize, Deserialize)]
pub struct GxIssuerCredSub {
    pub id: String,
    #[serde(rename = "gx:gaiaxTermsAndConditions")]
    pub gaiax_terms_and_conditions: String,
}
