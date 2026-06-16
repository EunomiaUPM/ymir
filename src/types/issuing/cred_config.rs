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

use std::collections::HashMap;

use crate::types::keys::Alg;
use crate::types::vcs::{VcFormat, VcType, VcTypeConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialConfiguration {
    pub format: String,
    pub cryptographic_binding_methods_supported: Vec<String>,
    pub credential_signing_alg_values_supported: Vec<String>,
    pub credential_definition: CredentialDefinition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDefinition {
    pub r#type: Vec<String>,
}

impl CredentialConfiguration {
    pub fn basic(vcs: Option<&[VcType]>) -> HashMap<String, CredentialConfiguration> {
        let vcs: Vec<VcType> = vcs
            .map(|s| s.to_vec())
            .unwrap_or_else(|| VcType::supported());

        let formats = VcFormat::supported();

        let alg: Vec<String> = Alg::supported()
            .into_iter()
            .map(|alg| alg.to_string())
            .collect();

        let mut credential_configurations_supported = HashMap::new();

        for vc_type in &vcs {
            for format in &formats {
                let config = VcTypeConfig::new(vc_type.clone(), format.clone());
                credential_configurations_supported.insert(
                    config.to_string(),
                    CredentialConfiguration {
                        format: config.format().to_string(),
                        cryptographic_binding_methods_supported: vec!["did".to_string()],
                        credential_signing_alg_values_supported: alg.clone(),
                        credential_definition: CredentialDefinition {
                            r#type: vec![
                                "VerifiableCredential".to_string(),
                                config.vc_type().to_string(),
                            ],
                        },
                    },
                );
            }
        }

        credential_configurations_supported
    }
}
