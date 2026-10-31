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

//! URN and URL helpers.

use std::str::FromStr;

use url::Url;
use urn::Urn;
use uuid::Uuid;

use crate::errors::{Errors, Outcome};

static UUID_PREFIX: &str = "urn:uuid:";

/// Returns the given URN, or a new `urn:uuid:<v4>`.
pub fn get_urn(optional_urn: Option<Urn>) -> Urn {
    optional_urn.unwrap_or_else(|| {
        let uuid = Uuid::new_v4();
        let id_string = format!("{UUID_PREFIX}{uuid}");

        id_string.parse::<Urn>().unwrap()
    })
}

/// New `urn:<prefix>:<v4>`.
pub fn generate_uuid_urn(prefix: &str) -> Urn {
    Urn::from_str(&format!("urn:{}:{}", prefix, uuid::Uuid::new_v4()))
        .expect("UUID URN is always valid")
}

/// Parses a string slice into a `Urn`.
pub fn parse_urn(s: &str) -> Outcome<Urn> {
    s.parse::<Urn>()
        .map_err(|e| Errors::crazy("invalid URN in database", Some(Box::new(e))))
}

/// Parses a string slice into a `Url`.
pub fn parse_url(s: &str) -> Outcome<Url> {
    Url::parse(s).map_err(|e| Errors::parse("Error parsing url", Some(Box::new(e))))
}

/// Parses a string slice into a `Urn` (backwards-compatible alias).
pub fn get_urn_from_string(string_in: &str) -> Outcome<Urn> {
    parse_urn(string_in)
}

/// Extension trait for parsing string slices into URNs.
pub trait ParseUrnExt {
    /// Parses the string slice into a `Urn`.
    fn parse_urn(&self) -> Outcome<Urn>;
}

impl ParseUrnExt for str {
    fn parse_urn(&self) -> Outcome<Urn> {
        parse_urn(self)
    }
}

#[cfg(test)]
mod tests;
