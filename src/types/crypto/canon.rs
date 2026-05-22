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

use crate::errors::Errors;
use serde_json::Value;

pub struct Canon {
    value: String,
}

impl TryFrom<&Value> for Canon {
    type Error = Errors;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let s = json_canon::to_string(value)
            .map_err(|e| Errors::parse("canonicalization failed", Some(Box::new(e))))?;
        Ok(Canon { value: s })
    }
}

impl AsRef<[u8]> for Canon {
    fn as_ref(&self) -> &[u8] {
        self.value.as_bytes()
    }
}

impl Canon {
    pub fn as_str(&self) -> &str {
        &self.value
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.value.as_bytes()
    }
}
