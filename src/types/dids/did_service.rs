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

use crate::impl_serde_via_str;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidService {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<DidServiceType>,
    r#type: String,
    #[serde(rename = "serviceEndpoint")]
    service_endpoint: String,
}

impl DidService {
    pub fn basic(r#type: DidServiceType, service_endpoint: String) -> Self {
        Self {
            id: None,
            r#type: r#type.to_string(),
            service_endpoint,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DidServiceType {
    AuthorizationServer,
    CredentialIssuer,
    FederatedCatalog,
    Other(String),
}

impl Display for DidServiceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            DidServiceType::AuthorizationServer => "AuthorizationServer",
            DidServiceType::CredentialIssuer => "CredentialIssuer",
            DidServiceType::FederatedCatalog => "FederatedCatalog",
            DidServiceType::Other(service) => service.as_str(),
        };

        write!(f, "{s}")
    }
}

impl FromStr for DidServiceType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AuthorizationServer" => Ok(DidServiceType::AuthorizationServer),
            "CredentialIssuer" => Ok(DidServiceType::CredentialIssuer),
            "FederatedCatalog" => Ok(DidServiceType::FederatedCatalog),
            other => Ok(DidServiceType::Other(other.to_owned())),
        }
    }
}

impl_serde_via_str!(DidServiceType);
