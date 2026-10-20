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

use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::Errors;

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum W3cDataModelVersion {
    V1,
    V2,
}

impl W3cDataModelVersion {
    pub fn context(&self) -> &'static str {
        match self {
            // W3cDataModelVersion::V1 => "https://www.w3.org/ns/credentials/v1",
            W3cDataModelVersion::V1 => "https://www.w3.org/2018/credentials/v1",
            W3cDataModelVersion::V2 => "https://www.w3.org/ns/credentials/v2",
        }
    }

    /// The version whose base context is in `contexts`, if any.
    pub fn from_context<S: AsRef<str>>(contexts: &[S]) -> Option<Self> {
        [W3cDataModelVersion::V2, W3cDataModelVersion::V1]
            .into_iter()
            .find(|v| contexts.iter().any(|c| c.as_ref() == v.context()))
    }

    /// The version of a JSON document, from its `@context` (a string or an array).
    pub fn from_document(doc: &serde_json::Value) -> Option<Self> {
        match doc.get("@context")? {
            serde_json::Value::String(c) => Self::from_context(&[c]),
            serde_json::Value::Array(cs) => {
                let cs: Vec<&str> = cs.iter().filter_map(|c| c.as_str()).collect();
                Self::from_context(&cs)
            }
            _ => None,
        }
    }
}

impl FromStr for W3cDataModelVersion {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "V1" => Ok(W3cDataModelVersion::V1),
            "V2" => Ok(W3cDataModelVersion::V2),
            format => Err(Errors::parse(
                format!("Invalid W3cDataModelVersion '{}'", format),
                None,
            )),
        }
    }
}

impl Display for W3cDataModelVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            W3cDataModelVersion::V1 => "V1",
            W3cDataModelVersion::V2 => "V2",
        };

        write!(f, "{s}")
    }
}

impl Default for W3cDataModelVersion {
    fn default() -> Self {
        W3cDataModelVersion::V2
    }
}
