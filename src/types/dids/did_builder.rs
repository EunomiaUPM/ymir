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

use super::{Did, JwkDidConfig, WebDidConfig};
use crate::errors::Outcome;
use crate::types::keys::PrivateKey;
use crate::utils::encode_url_safe_no_pad;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DidBuilder {
    Jwk(JwkDidConfig),
    Web(WebDidConfig),
}

impl DidBuilder {
    pub fn new_jwk(pem: impl Into<String>) -> DidBuilder {
        DidBuilder::Jwk(JwkDidConfig { pem: pem.into() })
    }
    pub fn new_web(domain: &str, path: Option<&str>, port: Option<&str>) -> DidBuilder {
        DidBuilder::Web(WebDidConfig {
            domain: domain.to_string(),
            path: path.map(|s| s.to_string()),
            port: port.map(|s| s.to_string()),
        })
    }
    pub fn build(&self) -> Outcome<Did> {
        let did = match self {
            DidBuilder::Jwk(JwkDidConfig { pem }) => {
                let key = PrivateKey::try_from_pkcs8_pem(pem)?;
                let jwk = serde_json::to_vec(&key.public_jwk())?;
                format!("did:jwk:{}", encode_url_safe_no_pad(jwk))
            }
            DidBuilder::Web(WebDidConfig { domain, path, port }) => {
                let mut did = format!("did:web:{domain}");
                if let Some(port) = port {
                    did = format!("{did}%3A{port}");
                }
                if let Some(path) = path {
                    did = format!("{did}:{}", path.replace('/', ":"));
                }
                did
            }
        };

        Did::parse(&did)
    }
}
