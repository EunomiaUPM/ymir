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
use std::sync::Arc;

use chrono::Utc;
use jsonwebtoken::{TokenData, Validation};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::info;

use crate::capabilities::DidResolver;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::utils::get_from_opt;

pub fn validate_data(node: &Value, field: &str) -> Outcome<String> {
    node.as_str()
        .ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                format!("Field '{}' is not a string", field),
                None,
            )
        })
        .map(|s| s.to_string())
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
    client: Arc<dyn ClientTrait>,
) -> Outcome<(TokenData<T>, String)>
where
    T: Serialize + DeserializeOwned,
{
    info!("Validating token");
    let header = jsonwebtoken::decode_header(&token).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            format!("Unable to decode token header: {}", token),
            Some(Box::new(e)),
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

    let token_data = jsonwebtoken::decode::<T>(&token, &key, &val)
        .map_err(|e| Errors::security("VPT signature is incorrect", Some(Box::new(e))))?;

    info!("Token signature is correct");
    Ok((token_data, kid.to_string()))
}
