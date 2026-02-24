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

use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use reqwest::Url;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::{Errors, Outcome};
use crate::types::errors::BadFormat;

pub fn get_from_opt<T>(value: Option<&T>, field_name: &str) -> Outcome<T>
where
    T: Clone + Serialize + DeserializeOwned
{
    value
        .ok_or_else(|| {
            Errors::format(BadFormat::Received, &format!("Missing field: {}", field_name), None)
        })
        .cloned()
}

pub fn trim_4_base(input: &str) -> String {
    let slashes: Vec<usize> = input.match_indices('/').map(|(i, _)| i).collect();

    if slashes.len() < 3 {
        return input.to_string();
    }

    let cut_index = slashes[2];

    input[..cut_index].to_string()
}

pub fn get_query_param(parsed_uri: &Url, param_name: &str) -> Outcome<String> {
    parsed_uri.query_pairs().find(|(k, _)| k == param_name).map(|(_, v)| v.into_owned()).ok_or_else(
        || {
            Errors::format(
                BadFormat::Received,
                format!("Missing query parameter '{}'", param_name),
                None
            )
        }
    )
}

pub fn json_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers
}
