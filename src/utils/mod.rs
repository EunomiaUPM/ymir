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

mod parse;

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use axum::http::HeaderMap;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, TokenData, Validation, encode};
pub use parse::*;
use rand::Rng;
use reqwest::Url;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::info;

use crate::capabilities::DidResolver;
use crate::errors::{Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::types::errors::BadFormat;

pub fn read<P>(path: P) -> Outcome<String>
where
    P: AsRef<Path>
{
    let path_ref = path.as_ref();

    fs::read_to_string(path_ref).map_err(|e| {
        Errors::read(
            path_ref.display().to_string(),
            format!("Unable to read file: {}", path_ref.display()),
            Some(anyhow::Error::from(e))
        )
    })
}

fn validate_data(node: &Value, field: &str) -> Outcome<String> {
    node.as_str()
        .ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                format!("Field '{}' is not a string", field),
                None
            )
        })
        .map(|s| s.to_string())
}

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

pub fn is_active(iat: u64) -> Outcome<()> {
    let now = Utc::now().timestamp() as u64;
    if now >= iat { Ok(()) } else { Err(Errors::forbidden("Token is not yet valid", None)) }
}

pub fn has_expired(exp: u64) -> Outcome<()> {
    let now = Utc::now().timestamp() as u64;
    if now <= exp { Ok(()) } else { Err(Errors::forbidden("Token has expired", None)) }
}

pub async fn validate_token<T>(
    token: &str,
    audience: Option<&str>,
    client: Arc<dyn ClientTrait>
) -> Outcome<(TokenData<T>, String)>
where
    T: Serialize + DeserializeOwned
{
    info!("Validating token");
    let header = jsonwebtoken::decode_header(&token).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            format!("Unable to decode token header: {}", token),
            Some(anyhow::Error::from(e))
        )
    })?;
    let kid_str = get_from_opt(header.kid.as_ref(), "kid")?;
    let alg = header.alg;

    let key = DidResolver::get_key(&kid_str, client).await?;
    let (kid, _) = DidResolver::split_did_id(&kid_str);

    let mut val = Validation::new(alg);

    val.required_spec_claims = HashSet::new();
    val.validate_exp = false;
    val.validate_nbf = true;

    match audience {
        Some(data) => {
            val.validate_aud = true;
            val.set_audience(&[&(data)]);
        }
        None => {
            val.validate_aud = false;
        }
    };

    let token_data = jsonwebtoken::decode::<T>(&token, &key, &val).map_err(|e| {
        Errors::security("VPT signature is incorrect", Some(anyhow::Error::from(e)))
    })?;

    info!("Token signature is correct");
    Ok((token_data, kid.to_string()))
}

pub fn trim_4_base(input: &str) -> String {
    let slashes: Vec<usize> = input.match_indices('/').map(|(i, _)| i).collect();

    if slashes.len() < 3 {
        return input.to_string();
    }

    let cut_index = slashes[2];

    input[..cut_index].to_string()
}

pub fn create_opaque_token() -> String {
    let mut bytes = [0u8; 32]; // 256 bits
    rand::rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

pub fn extract_gnap_token(headers: HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("GNAP "))
        .map(|token| token.to_string())
}

pub fn extract_bearer_token(headers: HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|token| token.to_string())
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

pub fn sign_token<T: Serialize>(header: &Header, claims: &T, key: &EncodingKey) -> Outcome<String> {
    encode(&header, &claims, &key).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            "Unable to sign token",
            Some(anyhow::Error::from(e))
        )
    })
}
