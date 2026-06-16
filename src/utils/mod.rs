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

mod client;
mod http;
mod parse;
mod token;

use reqwest::Url;
pub use client::http_client;
pub use http::*;
pub use parse::*;
pub use token::*;

use crate::errors::{BadFormat, Errors, Outcome};

pub fn require_field<T>(opt: Option<T>, field: &str) -> Outcome<T> {
    opt.ok_or_else(|| {
        Errors::missing_resource(
            field,
            format!("Required field '{}' is missing", field),
            None,
        )
    })
}

pub fn get_query_param(parsed_uri: &Url, param_name: &str) -> Outcome<String> {
    parsed_uri
        .query_pairs()
        .find(|(k, _)| k == param_name)
        .map(|(_, v)| v.into_owned())
        .ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                format!("Missing query parameter '{}'", param_name),
                None,
            )
        })
}