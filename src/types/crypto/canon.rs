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

use crate::errors::{Errors, Outcome};
use crate::utils::OneOrMany;
use serde::Serialize;

pub struct Canon {
    value: String,
    context: Option<OneOrMany<String>>,
}

impl Canon {
    pub fn new<T>(value: &T) -> Outcome<Self>
    where
        T: Serialize + ?Sized,
    {
        let json = serde_json::to_value(value)?;

        let context = json
            .get("@context")
            .map(|ctx| {
                serde_json::from_value::<OneOrMany<String>>(ctx.clone())
                    .map_err(|e| Errors::parse("invalid @context", Some(Box::new(e))))
            })
            .transpose()?;

        let value = json_canon::to_string(&json)
            .map_err(|e| Errors::parse("canonicalization failed", Some(Box::new(e))))?;
        Ok(Canon { value, context })
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
    pub fn context(&self) -> Option<&OneOrMany<String>> {
        self.context.as_ref()
    }
}
