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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::CredentialConfiguration;
use crate::types::vcs::VcType;

#[derive(Debug, Serialize, Deserialize)]
pub struct IssuerMetadata {
    pub issuer: String,
    pub credential_issuer: String,
    pub credential_endpoint: String,
    pub batch_credential_endpoint: String,
    pub jwks_uri: String,
    pub credential_configurations_supported: HashMap<String, CredentialConfiguration>,
    pub authorization_servers: Vec<String>,
}

impl IssuerMetadata {
    pub fn new(base_host: &str, host_path: &str, vcs: Option<&[VcType]>) -> Self {
        let credential_configurations_supported = CredentialConfiguration::basic(vcs);

        IssuerMetadata {
            issuer: base_host.to_string(),
            credential_issuer: base_host.to_string(),
            credential_endpoint: format!("{}/credential", host_path),
            batch_credential_endpoint: format!("{}/credential-batch", host_path),
            jwks_uri: format!("{}/jwks", host_path),
            credential_configurations_supported,
            authorization_servers: vec![base_host.to_string()],
        }
    }
}
