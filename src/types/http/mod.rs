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

use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

use crate::errors::{Errors, Outcome};

#[derive(Clone)]
pub enum HttpBody {
    Json(Value),
    Raw(String),
    Bytes(Vec<u8>),
    Form(HashMap<String, String>),
    None,
}

impl From<&str> for HttpBody {
    fn from(value: &str) -> Self {
        HttpBody::Raw(value.to_string())
    }
}

impl From<HashMap<String, String>> for HttpBody {
    fn from(value: HashMap<String, String>) -> Self {
        HttpBody::Form(value)
    }
}

impl HttpBody {
    pub fn json<T: Serialize>(value: &T) -> Outcome<HttpBody> {
        let body = serde_json::to_value(value)?;
        Ok(HttpBody::Json(body))
    }
    pub fn form<T: Serialize>(value: &T) -> Outcome<HttpBody> {
        let encoded = serde_urlencoded::to_string(value)
            .map_err(|e| Errors::parse("Failed to encode form", Some(Box::new(e))))?;
        let pairs: HashMap<String, String> = serde_urlencoded::from_str(&encoded)
            .map_err(|e| Errors::parse("Failed to decode form back", Some(Box::new(e))))?;
        Ok(HttpBody::Form(pairs))
    }

    pub fn str(value: &str) -> Outcome<HttpBody> {
        Ok(HttpBody::Raw(value.to_string()))
    }
    pub fn from_json_bytes<T: Serialize>(value: &T) -> Outcome<(HttpBody, Vec<u8>)> {
        let bytes = serde_json::to_vec(value)
            .map_err(|e| Errors::parse("Failed to serialize body to bytes", Some(Box::new(e))))?;
        Ok((HttpBody::Bytes(bytes.clone()), bytes))
    }
}
