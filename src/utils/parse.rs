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

use std::path::Path;
use std::{env, fs};

use axum::http::HeaderValue;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::EncodingKey;
use reqwest::Response;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::errors::{BadFormat, Errors, Outcome};
use crate::utils::validate_data;

pub trait ParseHeaderExt {
    fn parse_header(&self) -> Outcome<HeaderValue>;
}

impl ParseHeaderExt for str {
    fn parse_header(&self) -> Outcome<HeaderValue> {
        self.parse().map_err(|e| {
            Errors::parse(format!("Invalid header value: '{}'", self), Some(Box::new(e)))
        })
    }
}

pub async fn parse_json_resp<T: DeserializeOwned>(response: Response) -> Outcome<T> {
    response
        .json()
        .await
        .map_err(|e| Errors::parse("Unable to parse json response", Some(Box::new(e))))
}

pub async fn parse_text_resp(response: Response) -> Outcome<String> {
    response
        .text()
        .await
        .map_err(|e| Errors::parse("Unable to parse text response", Some(Box::new(e))))
}

pub fn parse_from_value<T: DeserializeOwned>(value: Value) -> Outcome<T> {
    serde_json::from_value(value)
        .map_err(|e| Errors::parse("Unable to parse from value", Some(Box::new(e))))
}

pub fn parse_from_str<T: DeserializeOwned>(data: &str) -> Outcome<T> {
    serde_json::from_str(data)
        .map_err(|e| Errors::parse("Unable to parse from value", Some(Box::new(e))))
}

pub fn parse_from_slice<T: DeserializeOwned>(data: &[u8]) -> Outcome<T> {
    serde_json::from_slice(data)
        .map_err(|e| Errors::parse("Unable to parse from slice", Some(Box::new(e))))
}

pub fn parse_to_value<T: Serialize>(value: &T) -> Outcome<Value> {
    serde_json::to_value(value)
        .map_err(|e| Errors::parse("Unable to serialize value", Some(Box::new(e))))
}

pub fn parse_to_string<T: Serialize>(value: &T) -> Outcome<String> {
    serde_json::to_string(value)
        .map_err(|e| Errors::parse("Unable to parse to string", Some(Box::new(e))))
}

pub fn get_rsa_key(key: &str) -> Outcome<EncodingKey> {
    EncodingKey::from_rsa_pem(key.as_bytes())
        .map_err(|e| Errors::parse("Unable to decode RSA key", Some(Box::new(e))))
}

pub fn decode_url_safe_no_pad(data: &str) -> Outcome<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(data).map_err(|e| {
        Errors::parse(
            format!("Unable to decode url safe no pad: {}", data),
            Some(Box::new(e)),
        )
    })
}

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

pub fn read_json<T, P>(path: P) -> Outcome<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let data = read(path)?;
    serde_json::from_str(&data)
        .map_err(|e| Errors::parse("Unable to parse JSON from file", Some(Box::new(e))))
}

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

pub fn write_json<T, P>(path: P, value: &T) -> Outcome<()>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let data = serde_json::to_string_pretty(value)
        .map_err(|e| Errors::parse("Unable to serialize value to JSON", Some(Box::new(e))))?;
    write(path, data)
}

pub fn expect_from_env(env: &str) -> String {
    env::var(env).expect(format!("Environment variable {} not set", env).as_str())
}

pub fn get_claim(claims: &Value, path: &[&str]) -> Outcome<String> {
    let mut node = claims;
    let field = path.last().ok_or_else(|| Errors::parse("Path vector is empty", None))?;
    for key in path.iter() {
        node = node.get(key).ok_or_else(|| {
            Errors::format(BadFormat::Received, format!("Missing field '{}'", key), None)
        })?
    }
    validate_data(node, field)
}

pub fn get_opt_claim(claims: &Value, path: &[&str]) -> Outcome<Option<String>> {
    let mut node = claims;
    let field = path.last().ok_or_else(|| Errors::parse("Path vector is empty", None))?;
    for key in path.iter() {
        node = match node.get(key) {
            Some(data) => data,
            None => return Ok(None),
        };
    }
    let data = validate_data(node, field)?;
    Ok(Some(data))
}
