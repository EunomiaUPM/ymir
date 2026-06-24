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

use std::path::Path;
use std::{env, fs};

use axum::http::HeaderValue;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::errors::{BadFormat, Errors, Outcome};

// ===== AXUM LAYER EXTENSIONS =====================================================================

/// Trait provisioning fast conversion hooks from text boundaries to axum network [`HeaderValue`] structures.
pub trait ParseHeaderExt {
    /// Parses a raw string slice into a validated axum network [`HeaderValue`].
    ///
    /// # Errors
    /// Returns an [`Errors::ParseError`] if the data payload contains illegal characters.
    fn parse_header(&self) -> Outcome<HeaderValue>;
}

impl ParseHeaderExt for str {
    fn parse_header(&self) -> Outcome<HeaderValue> {
        self.parse().map_err(|e| {
            Errors::parse(
                format!("Invalid header value: '{}'", self),
                Some(Box::new(e)),
            )
        })
    }
}

// ===== BASE64URL REWIRING ENGINE =================================================================

/// Decodes an unpadded, URL-safe Base64 encoded payload back into raw vector bytes.
pub fn decode_url_safe_no_pad(data: &str) -> Outcome<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(data).map_err(|e| {
        Errors::parse(
            format!("Unable to decode url safe no pad: {}", data),
            Some(Box::new(e)),
        )
    })
}

/// Encodes an arbitrary byte matrix array slice into an unpadded, URL-safe Base64 string literal.
pub fn encode_url_safe_no_pad(data: impl AsRef<[u8]>) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

// ===== FILESYSTEM RAW STORAGE PIPELINES ==========================================================

/// Reads a local target asset track from disk into an unstructured textual string buffer.
pub fn read<P>(path: P) -> Outcome<String>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();

    fs::read_to_string(path_ref).map_err(|e| {
        Errors::read(
            path_ref.display().to_string(),
            format!("Unable to read file: {}", path_ref.display()),
            Some(Box::new(e)),
        )
    })
}

/// Dispatches a standard serialized string output stream over a targeted filesystem layout vector.
pub fn write<P>(path: P, content: String) -> Outcome<()>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();

    fs::write(path_ref, content).map_err(|e| {
        Errors::write(
            path_ref.display().to_string(),
            format!("Unable to write file: {}", path_ref.display()),
            Some(Box::new(e)),
        )
    })
}

// ===== SERIALIZED JSON FILE WRAPPERS =============================================================

/// Reads a text configuration asset from disk, marshalling its parameters into structured models `T`.
pub fn read_json<T, P>(path: P) -> Outcome<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let data = read(path)?;
    serde_json::from_str(&data)
        .map_err(|e| Errors::parse("Unable to parse JSON from file", Some(Box::new(e))))
}

/// Transforms data matrices into pretty-printed JSON configurations before writing them to disk.
pub fn write_json<T, P>(path: P, value: &T) -> Outcome<()>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let data = serde_json::to_string_pretty(value)
        .map_err(|e| Errors::parse("Unable to serialize value to JSON", Some(Box::new(e))))?;
    write(path, data)
}

// ===== SYSTEM ENVIRONMENT UTILITIES ==============================================================

/// Forces a synchronous system variable resolution hook against host system scopes.
///
/// # Panics
/// Direct unrecoverable panic occurs if the targeted environment token identifier remains unassigned.
pub fn expect_from_env(env: &str) -> String {
    env::var(env).expect(&format!("Environment variable {} not set", env))
}

// ===== DATA VALIDATION & STRUCTURAL EXTRACTORS ===================================================

/// Extracts a single parameter field boundary mapping query out of an unstructured parsed [`Url`].
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

/// Evaluates options arrays, unwrapping raw targets or generating structured resource missing track errors.
pub fn require_field<T>(opt: Option<T>, field: &str) -> Outcome<T> {
    opt.ok_or_else(|| {
        Errors::missing_resource(
            field,
            format!("Required field '{}' is missing", field),
            None,
        )
    })
}

// ===== SHARED POLYMORPHIC DATA TYPES =============================================================

/// Structural multi-format container mapping parameters that accept either solitary strings or raw string arrays.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArr {
    /// Singular text
    String(String),
    /// Array
    Arr(Vec<String>),
}