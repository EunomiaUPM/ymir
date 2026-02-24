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

use jsonwebtoken::DecodingKey;
use jsonwebtoken::jwk::Jwk;
use serde_json::Value;
use tracing::info;

use crate::errors::{Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::types::dids::did_type::DidType;
use crate::types::errors::BadFormat;
use crate::utils::{
    decode_url_safe_no_pad, json_headers, parse_from_slice, parse_from_value, parse_json_resp
};

pub struct DidResolver;

impl DidResolver {
    pub fn split_did_id(did: &str) -> (&str, Option<&str>) {
        match did.split_once('#') {
            Some((did_kid, id)) => (did_kid, Some(id)),
            None => (did, None)
        }
    }

    pub async fn get_key(did: &str, client: Arc<dyn ClientTrait>) -> Outcome<DecodingKey> {
        info!("Retrieving key from did");
        let (did_base, kid_opt) = DidResolver::split_did_id(did);

        let key: Jwk = match Self::parse_did(did) {
            DidType::Jwk => {
                let vec = decode_url_safe_no_pad(&(did_base.replace("did:jwk:", "")))?;
                let jwk: Jwk = parse_from_slice(&vec)?;
                jwk
            }

            DidType::Web => {
                let domain = did_base.replace("did:web:", "");
                let url = Self::parse_domain(&domain);

                info!("Resolving DID Document: {}", url);

                let headers = json_headers();

                let res = client.get(&url, Some(headers)).await?;

                let doc: Value = match res.status().as_u16() {
                    200 => {
                        info!("DID Document retrieved successfully");

                        parse_json_resp(res).await?
                    }
                    status => {
                        return Err(Errors::petition(
                            url,
                            "GET",
                            Some(status),
                            "Didi Document not retrieved",
                            None
                        ));
                    }
                };

                let methods = doc["verificationMethod"].as_array().ok_or_else(|| {
                    Errors::format(BadFormat::Received, "Missing verificationMethod", None)
                })?;

                let method = if let Some(kid) = kid_opt {
                    let full_kid = format!("{}#{}", did_base, kid);
                    methods
                        .iter()
                        .find(|m| m["id"] == full_kid)
                        .ok_or_else(|| Errors::format(BadFormat::Received, "Key not found", None))?
                } else {
                    methods.first().ok_or_else(|| {
                        Errors::format(
                            BadFormat::Received,
                            "No verification methods in DID Document",
                            None
                        )
                    })?
                };

                let jwk_value = method["publicKeyJwk"].as_object().ok_or_else(|| {
                    Errors::format(BadFormat::Received, "Missing publicKeyJwk", None)
                })?;

                let jwk: Jwk = parse_from_value(jwk_value.clone().into())?;

                jwk
            }

            DidType::Other => return Err(Errors::not_impl(format!("Did method: {}", did), None))
        };
        DecodingKey::from_jwk(&key).map_err(|e| {
            Errors::parse("Error parsing decoding key to jwk", Some(anyhow::Error::from(e)))
        })
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
