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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::errors::{BadFormat, Errors, Outcome, PetitionFailure};
use crate::services::client::ClientTrait;
use crate::types::dids::{DidBuilder, DidDocument, DidType, JwkDid, VerificationMaterial, WebDid};
use crate::types::keys::{Crv, Key, KeyData, Kty, RetrievedKey, RetrievedKeyData};
use crate::utils::{HasId, ResponseExt, decode_url_safe_no_pad, http_client};
use serde_json::Value;
use std::str::FromStr;

pub enum Did {
    Jwk(JwkDid),
    Web(WebDid),
}

impl HasId for Did {
    fn id(&self) -> &str {
        match self {
            Did::Jwk(j) => &j.id(),
            Did::Web(w) => &w.id(),
        }
    }
}

impl Did {
    pub fn parse_from_kid(kid: &str) -> Outcome<Did> {
        let (did, key_id) = kid.split_once('#').map(|(did, key_id)| (did, Some(key_id.to_string()))).unwrap_or((kid, None));
        if let Some(rest) = did.strip_prefix("did:web:") {
            let parts: Vec<&str> = rest.split(':').collect();

            let (domain, path) = match parts.as_slice() {
                [domain] => (domain.to_owned(), None),
                [domain, path @ ..] => {
                    let path = path.join("/");
                    (domain.to_owned(), Some(path))
                }
                _ => {
                    return Err(Errors::format(
                        BadFormat::Received,
                        "Invalid DID format",
                        None,
                    ));
                }
            };

            let (domain, port) = match domain.split_once("%3A") {
                Some((domain, port)) => (domain, Some(port.to_owned())),
                None => (domain, None),
            };
            let w = WebDid::new(did, kid, domain.to_owned(), path, port, key_id);

            Ok(Did::Web(w))
        } else if let Some(rest) = did.strip_prefix("did:jwk:") {
            let j = JwkDid::new(did, kid, rest.to_owned(), key_id);

            Ok(Did::Jwk(j))
        } else {
            Err(Errors::not_impl(
                format!("Did format {kid} not supported"),
                None,
            ))
        }
    }

    pub async fn get_key(&self) -> Outcome<RetrievedKey> {
        let data = match self {
            Did::Jwk(jwk) => Self::resolve_jwk(&jwk)?,
            Did::Web(web) => Self::resolve_web(&web).await?,
        };

        Ok(RetrievedKey {
            did: self.id().to_string(),
            data,
        })
    }

    fn resolve_jwk(jwk_did: &JwkDid) -> Outcome<RetrievedKeyData> {
        let jwk_bytes = decode_url_safe_no_pad(&jwk_did.jwk())?;
        let jwk_json: Value = serde_json::from_slice(&jwk_bytes)?;
        Self::resolve_value(&jwk_json)
    }

    async fn resolve_web(web_did: &WebDid) -> Outcome<RetrievedKeyData> {
        let url = Self::get_web_url(web_did);

        let resp = http_client().get(&url, None).await?;

        if !resp.status().is_success() {
            return Err(Errors::petition(
                &url,
                "GET",
                Some(resp.status().clone()),
                PetitionFailure::HttpStatus(resp.status()),
                "Error retrieving did:web document",
                None,
            ));
        }

        let did_doc: DidDocument = resp.parse_json().await?;

        // TEMPORAL: en vez de localizar el VM por `<did>#<fragment>`
        // (que algunos firmantes hoy generan con fragment vacío, p.ej.
        // `BasicIssuerService::get_sig_context` que construye la `Key`
        // con id `""`), cogemos el primer VM del doc. Suficiente para
        // docs single-VM; revisitar cuando el `kid` lleve fragment
        // estable.
        let method = did_doc
            .verification_method
            .first()
            .ok_or_else(|| {
                Errors::format(
                    BadFormat::Received,
                    format!(
                        "DID document for {} has no verificationMethod",
                        web_did.id()
                    ),
                    None,
                )
            })?;

        match &method.material {
            VerificationMaterial::JsonWebKey { public_key_jwk } => {
                Self::resolve_value(&public_key_jwk)
            }
            VerificationMaterial::JsonWebKey2020 { public_key_jwk } => {
                Self::resolve_value(&public_key_jwk)
            }
            VerificationMaterial::Multikey { .. } => {
                Err(Errors::not_impl(
                    "Multikey type for verification method not implemented",
                    None,
                ))
                // todo!()
            }
        }
    }

    fn resolve_value(value: &Value) -> Outcome<RetrievedKeyData> {
        let kty_str = value["kty"]
            .as_str()
            .ok_or_else(|| Errors::parse("JWK missing kty", None))?;
        let Ok(kty) = Kty::from_str(kty_str);

        let crv = value
            .get("crv")
            .and_then(|v| v.as_str())
            .map(|s| Crv::from_str(s).unwrap());

        match (&kty, crv.as_ref()) {
            (Kty::Okp, Some(Crv::Ed25519)) => RetrievedKeyData::build_ed25519_data(&value),
            (Kty::Rsa, _) => RetrievedKeyData::build_rsa_data(&value),
            _ => {
                let crv = if let Some(crv) = crv {
                    crv.to_string()
                } else {
                    "".to_string()
                };
                Err(Errors::not_impl(
                    format!("unsupported JWK: kty={} crv={:?}", kty, crv),
                    None,
                ))
            }
        }
    }

    fn get_web_url(did: &WebDid) -> String {
        let port = match did.port().as_ref() {
            Some(port) => format!(":{port}"),
            None => "".to_string(),
        };
        if let Some(path) = &did.path() {
            format!("https://{}{}/{}/did.json", did.domain(), port, path)
        } else {
            format!("https://{}{}/.well-known/did.json", did.domain(), port)
        }
    }
    pub fn create(builder: DidBuilder, keys: &[Key]) -> Outcome<Did> {
        match builder {
            DidBuilder::Jwk => {
                let key = keys.first().ok_or_else(|| {
                    Errors::format(
                        BadFormat::Sent,
                        "did:jwk requires at least one key",
                        None,
                    )
                })?;
                let did = key.data().to_did_jwk()?;
                Did::parse_from_kid(&did)
            }
            DidBuilder::Web(web) => Ok(Did::Web(web)),
        }
    }
    pub fn r#type(&self) -> DidType {
        match self {
            Did::Jwk(_) => DidType::Jwk,
            Did::Web(_) => DidType::Web,
        }
    }
}
