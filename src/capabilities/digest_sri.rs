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

use crate::errors::{Errors, Outcome};
use crate::types::crypto::Canon;
use base64::{Engine, engine::general_purpose};
use sha2::{Digest, Sha256};

pub struct DigestSRI;

impl DigestSRI {
    pub fn digest(canonical: &Canon) -> String {
        let hash = Sha256::digest(canonical.as_ref());
        let b64 = general_purpose::STANDARD.encode(hash);
        format!("sha256-{}", b64)
    }

    pub fn validate_json_sri(canonical: &Canon, sri: impl Into<String>) -> Outcome<bool> {
        let sri = sri.into();

        if !sri.starts_with("sha256-") {
            return Err(Errors::not_impl(
                "Digest SRI only accepts sha 256 right now",
                None,
            ));
        }

        let computed = Self::digest(canonical);
        Ok(computed == sri)
    }
}
