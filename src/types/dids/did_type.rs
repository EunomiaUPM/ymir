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

use crate::errors::Errors;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DidType {
    Jwk,
    Web,
}

impl Display for DidType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            DidType::Jwk => "Jwk",
            DidType::Web => "Web",
        };
        write!(f, "{s}")
    }
}

impl FromStr for DidType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Jwk" => Ok(DidType::Jwk),
            "Web" => Ok(DidType::Web),
            did => Err(Errors::not_impl(
                format!("DidType {did} not supported"),
                None,
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JwkDid {
    id: String,
    jwk: String,
}

impl JwkDid {
    pub fn new(id: impl Into<String>, jwk: impl Into<String>) -> JwkDid {
        JwkDid { id: id.into(), jwk: jwk.into() }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn jwk(&self) -> &str {
        &self.jwk
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebDid {
    id: String,
    domain: String,
    path: Option<String>,
    port: Option<String>,
}

impl WebDid {
    pub fn new(id: impl Into<String>, domain: impl Into<String>, path: Option<String>, port: Option<String>) -> WebDid {
        WebDid {
            id: id.into(),
            domain: domain.into(),
            path,
            port,
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn domain(&self) -> &str {
        &self.domain
    }
    pub fn path(&self) -> &Option<String> {
        &self.path
    }
    pub fn port(&self) -> &Option<String> {
        &self.port
    }

    pub fn get_web_url(&self) -> String {
        let port = match self.port().as_ref() {
            Some(port) => format!(":{port}"),
            None => "".to_string(),
        };
        if let Some(path) = &self.path() {
            format!("https://{}{}/{}/did.json", self.domain(), port, path)
        } else {
            format!("https://{}{}/.well-known/did.json", self.domain(), port)
        }
    }
}
