/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::errors::Outcome;
use crate::utils::parse_to_value;

#[derive(Clone)]
pub enum Body {
    Json(Value),
    Raw(String),
    Form(HashMap<String, String>),
    None
}

impl From<&str> for Body {
    fn from(value: &str) -> Self { Body::Raw(value.to_string()) }
}

impl From<HashMap<String, String>> for Body {
    fn from(value: HashMap<String, String>) -> Self { Body::Form(value) }
}

impl Body {
    pub fn json<T: Serialize>(value: &T) -> Outcome<Body> {
        let body = parse_to_value(value)?;
        Ok(Body::Json(body))
    }

    pub fn str(value: &str) -> Outcome<Body> { Ok(Body::Raw(value.to_string())) }
}
