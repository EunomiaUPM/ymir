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

use async_trait::async_trait;
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use axum::{Form, Json};
use reqwest::{Response, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::{BadFormat, Errors, Outcome, PetitionFailure};

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

#[async_trait]
pub trait ResponseExt {
    async fn parse_json<T: DeserializeOwned>(self) -> Outcome<T>;
    async fn parse_text(self) -> Outcome<String>;
}

#[async_trait]
impl ResponseExt for Response {
    async fn parse_json<T: DeserializeOwned>(self) -> Outcome<T> {
        let url = self.url().to_string();
        let status = self.status();
        let text = self.parse_text().await?;

        serde_json::from_str(&text).map_err(|e| {
            Errors::petition(
                &url,
                "unknown",
                Some(status),
                PetitionFailure::BodyDeserialization,
                format!("Raw: {}", text),
                Some(Box::new(e))
            )
        })
    }

    async fn parse_text(self) -> Outcome<String> {
        let url = self.url().to_string();
        let status = self.status();
        self.text().await.map_err(|e| {
            Errors::petition(
                &url,
                "unknown",
                Some(status),
                PetitionFailure::BodyRead,
                "Failed to read body",
                Some(Box::new(e))
            )
        })
    }
}

pub fn extract_payload<T>(payload: Result<Json<T>, JsonRejection>) -> Outcome<T> {
    payload.map(|Json(v)| v).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            "Error extracting Json payload",
            Some(Box::new(e))
        )
    })
}

pub fn extract_form_payload<T>(payload: Result<Form<T>, FormRejection>) -> Outcome<T> {
    payload.map(|Form(v)| v).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            "Error extracting form payload",
            Some(Box::new(e))
        )
    })
}

pub fn extract_query_param(params: &HashMap<String, String>, key: &str) -> Outcome<String> {
    params.get(key).cloned().ok_or_else(|| {
        Errors::format(
            BadFormat::Received,
            format!("Unable to retrieve '{}' from query params", key),
            None
        )
    })
}

pub fn extract_gnap_token(headers: HeaderMap) -> Outcome<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("GNAP "))
        .map(|token| token.to_string())
        .ok_or_else(|| Errors::unauthorized("GNAP token missing", None))
}

pub fn extract_bearer_token(headers: HeaderMap) -> Outcome<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|token| token.to_string())
        .ok_or_else(|| Errors::unauthorized("Bearer token missing", None))
}
