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

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::{env, fs};

use anyhow::bail;
use axum::http::HeaderMap;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, TokenData, Validation, encode};
use rand::Rng;
use reqwest::Url;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::{error, info};

use crate::capabilities::DidResolver;
use crate::errors::{ErrorLogTrait, Errors};
use crate::services::client::ClientTrait;
use crate::types::errors::BadFormat;

pub fn read<P>(path: P) -> anyhow::Result<String>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();
    match fs::read_to_string(path_ref) {
        Ok(data) => Ok(data),
        Err(e) => {
            let error = Errors::read_new(&path_ref.display().to_string(), &e.to_string());
            error!("{}", error.log());
            bail!(error)
        }
    }
}

pub fn read_json<T, P>(path: P) -> anyhow::Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let data = read(path)?;
    let json = serde_json::from_str(&data)?;
    Ok(json)
}

pub fn expect_from_env(env: &str) -> String {
    let result = env::var(env);
    let data = match result {
        Ok(data) => Some(data),
        Err(e) => {
            let error = Errors::env_new(format!("{} not found -> {}", &env, e.to_string()));
            error!("{}", error.log());
            None
        }
    };
    data.expect("Error with env variable")
}

pub fn get_claim(claims: &Value, path: Vec<&str>) -> anyhow::Result<String> {
    let mut node = claims;
    let field = path.last().unwrap_or(&"unknown");
    for key in path.iter() {
        node = match node.get(key) {
            Some(data) => data,
            None => {
                let error =
                    Errors::format_new(BadFormat::Received, &format!("Missing field '{}'", key));
                error!("{}", error.log());
                bail!(error)
            }
        };
    }
    validate_data(node, field)
}

pub fn get_opt_claim(claims: &Value, path: Vec<&str>) -> anyhow::Result<Option<String>> {
    let mut node = claims;
    let field = path.last().unwrap_or(&"unknown");
    for key in path.iter() {
        node = match node.get(key) {
            Some(data) => data,
            None => return Ok(None),
        };
    }
    let data = validate_data(node, field)?;
    Ok(Some(data))
}

fn validate_data(node: &Value, field: &str) -> anyhow::Result<String> {
    match node.as_str() {
        Some(data) => Ok(data.to_string()),
        None => {
            let error =
                Errors::format_new(BadFormat::Received, &format!("Field '{}' not a string", field));
            error!("{}", error.log());
            bail!(error)
        }
    }
}

pub fn get_from_opt<T>(value: &Option<T>, field_name: &str) -> anyhow::Result<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    match value {
        Some(v) => Ok(v.clone()),
        None => {
            let error = Errors::unauthorized_new(&format!("Missing field: {}", field_name));
            error!("{}", error.log());
            bail!(error);
        }
    }
}

pub fn is_active(iat: u64) -> anyhow::Result<()> {
    let now = Utc::now().timestamp() as u64;
    if now >= iat {
        Ok(())
    } else {
        let error = Errors::forbidden_new("Token is not yet valid");
        error!("{}", error.log());
        bail!(error);
    }
}

pub fn has_expired(exp: u64) -> anyhow::Result<()> {
    let now = Utc::now().timestamp() as u64;
    if now <= exp {
        Ok(())
    } else {
        let error = Errors::forbidden_new("Token has expired");
        error!("{}", error.log());
        bail!(error);
    }
}

pub async fn validate_token<T>(
    token: &str,
    audience: Option<&str>,
    client: Arc<dyn ClientTrait>,
) -> anyhow::Result<(TokenData<T>, String)>
where
    T: Serialize + DeserializeOwned,
{
    info!("Validating token");
    let header = jsonwebtoken::decode_header(&token)?;
    info!("{:#?}", header);
    let kid_str = get_from_opt(&header.kid, "kid")?;
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

    let token_data = match jsonwebtoken::decode::<T>(&token, &key, &val) {
        Ok(data) => data,
        Err(e) => {
            let error =
                Errors::security_new(&format!("VPT signature is incorrect -> {}", e.to_string()));
            error!("{}", error.log());
            bail!(error);
        }
    };

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

pub fn get_query_param(parsed_uri: &Url, param_name: &str) -> anyhow::Result<String> {
    if let Some(value) =
        parsed_uri.query_pairs().find(|(k, _)| k == param_name).map(|(_, v)| v.into_owned())
    {
        Ok(value)
    } else {
        let error = Errors::format_new(
            BadFormat::Received,
            &format!("The expected '{}' field was missing in the oidc4vp uri", param_name),
        );
        error!("{}", error.log());
        bail!(error);
    }
}

pub fn sign_token<T: Serialize>(
    header: &Header,
    claims: &T,
    key: &EncodingKey,
) -> anyhow::Result<String> {
    let data = encode(&header, &claims, &key).map_err(|e| {
        let error = Errors::format_new(
            BadFormat::Unknown,
            &format!("Error signing token: {}", e.to_string()),
        );
        error!("{}", error.log());
        error
    })?;
    Ok(data)
}

pub fn get_rsa_key(key: String) -> anyhow::Result<EncodingKey> {
    let data = EncodingKey::from_rsa_pem(key.as_bytes()).map_err(|e| {
        let error = Errors::format_new(
            BadFormat::Unknown,
            &format!("Error parsing private key: {}", e.to_string()),
        );
        error!("{}", error.log());
        error
    })?;
    Ok(data)
}
