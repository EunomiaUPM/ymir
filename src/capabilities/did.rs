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

use std::sync::Arc;

use anyhow::bail;
use axum::http::HeaderMap;
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::jwk::Jwk;
use serde_json::Value;
use tracing::{error, info};

use crate::errors::{ErrorLogTrait, Errors};
use crate::services::client::ClientTrait;
use crate::types::dids::did_type::DidType;
use crate::types::errors::BadFormat;

pub struct DidResolver;

impl DidResolver {
    pub fn split_did_id(did: &str) -> (&str, Option<&str>) {
        match did.split_once('#') {
            Some((did_kid, id)) => (did_kid, Some(id)),
            None => (did, None)
        }
    }

    pub async fn get_key(did: &str, client: Arc<dyn ClientTrait>) -> anyhow::Result<DecodingKey> {
        info!("Retrieving key from did");
        let key = match Self::parse_did(did) {
            DidType::Jwk => {
                let (did_base, _) = DidResolver::split_did_id(did);

                let vec = URL_SAFE_NO_PAD.decode(&(did_base.replace("did:jwk:", "")))?;
                let jwk: Jwk = serde_json::from_slice(&vec)?;

                DecodingKey::from_jwk(&jwk)?
            }
            DidType::Web => {
                let (did_base, kid) = DidResolver::split_did_id(did);

                let kid = kid.ok_or_else(|| {
                    let error = Errors::format_new(
                        BadFormat::Received,
                        "did:web requires a key id fragment (#...)"
                    );
                    error!("{}", error.log());
                    error
                })?;

                let domain = did_base.replace("did:web:", "");

                let url = Self::parse_domain(&domain);

                info!("Resolving DID Document: {}", url);

                let mut headers = HeaderMap::new();
                headers.insert(CONTENT_TYPE, "application/json".parse()?);
                headers.insert(ACCEPT, "application/json".parse()?);

                let res = client.get(&url, Some(headers)).await?;

                let doc: Value = match res.status().as_u16() {
                    200 => {
                        info!("Did Document retrieved successfully retrieved");
                        res.json().await?
                    }
                    _ => {
                        let error =
                            Errors::format_new(BadFormat::Received, "Did Document not retrieved");
                        error!("{}", error.log());
                        bail!(error)
                    }
                };

                let methods = doc["verificationMethod"].as_array().ok_or_else(|| {
                    let error =
                        Errors::format_new(BadFormat::Received, "Missing verification method");
                    error!("{}", error.log());
                    error
                })?;

                let full_kid = format!("{}#{}", did_base, kid);

                let method = methods.iter().find(|m| m["id"] == full_kid).ok_or_else(|| {
                    let error = Errors::format_new(
                        BadFormat::Received,
                        &format!("Key not found: {}", full_kid)
                    );
                    error!("{}", error.log());
                    error
                })?;

                let jwk_value = method["publicKeyJwk"].as_object().ok_or_else(|| {
                    let error = Errors::format_new(BadFormat::Received, "Missing publicKeyJwk");
                    error!("{}", error.log());
                    error
                })?;

                let jwk: Jwk = serde_json::from_value(jwk_value.clone().into())?;

                DecodingKey::from_jwk(&jwk)?
            }
            DidType::Other => {
                let error = Errors::not_impl_new("did method", &did.to_string());
                error!("{}", error.log());
                bail!(error);
            }
        };
        Ok(key)
    }

    pub fn parse_did(did: &str) -> DidType {
        if did.starts_with("did:web:") {
            DidType::Web
        } else if did.starts_with("did:jwk:") {
            DidType::Jwk
        } else {
            DidType::Other
        }
    }
    pub fn parse_domain(domain: &str) -> String {
        let parts: Vec<&str> = domain.split(':').collect();

        match parts.as_slice() {
            [domain] => format!("https://{}/.well-known/did.json", domain),
            [domain, path @ ..] => {
                let path = path.join("/");
                format!("https://{}/{}/did.json", domain, path)
            }
            _ => String::new()
        }
    }
}
