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

use crate::errors::Errors;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidService {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
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

pub enum DidServiceType {
    AuthorizationServer,
    CredentialIssuer,
}

impl Display for DidServiceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            DidServiceType::AuthorizationServer => "AuthorizationServer",
            DidServiceType::CredentialIssuer => "CredentialIssuer",
        };

        write!(f, "{s}")
    }
}

impl FromStr for DidServiceType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AuthorizationServer" => Ok(DidServiceType::AuthorizationServer),
            "CredentialIssuer" => Ok(DidServiceType::CredentialIssuer),
            format => Err(Errors::parse(
                format!("Unknown service type: {}", format),
                None,
            )),
        }
    }
}
