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

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::types::vcs::{VcFormat, VcType, VcTypeConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct VcCredOffer {
    pub credential_issuer: String,
    pub grants: CredOfferGrants,
    pub credential_configuration_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredOfferGrants {
    #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
    pub urn_pre_authorized_code: UrnPreAuthorizedCode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrnPreAuthorizedCode {
    #[serde(rename = "pre-authorized_code")]
    pub pre_authorized_code: String,
}

impl VcCredOffer {
    pub fn new<S: Into<String>, T: Into<String>>(
        issuer: S,
        token: T,
        vc_types: &[VcType],
    ) -> Self {
        let formats = VcFormat::supported();
        let mut configuration_ids: Vec<String> = Vec::new();

        for vc_type in vc_types {
            for format in &formats {
                let config = VcTypeConfig::new(vc_type.clone(), format.clone());
                configuration_ids.push(config.to_string());
            }
        }

        VcCredOffer {
            credential_issuer: issuer.into(),
            grants: CredOfferGrants {
                urn_pre_authorized_code: UrnPreAuthorizedCode {
                    pre_authorized_code: token.into(),
                },
            },
            credential_configuration_ids: configuration_ids,
        }
    }
}
