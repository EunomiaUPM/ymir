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

use crate::types::keys::Alg;
use crate::types::vcs::{VcType, W3cDataModelVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputDescriptor {
    pub id: String,
    pub format: InputDescriptorFormat,
    pub constraints: InputDescriptorConstraints,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputDescriptorFormat {
    jwt_vc_json: InputDescriptorFormatJWTJson,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputDescriptorFormatJWTJson {
    pub alg: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputDescriptorConstraints {
    pub fields: Vec<InputDescriptorConstraintsFields>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputDescriptorConstraintsFields {
    pub path: Vec<String>,
    pub filter: InputDescriptorConstraintsFieldsFilter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputDescriptorConstraintsFieldsFilter {
    pub r#type: String,
    pub pattern: String,
}

impl InputDescriptor {
    pub fn new(vc_type: &VcType, model: W3cDataModelVersion) -> Self {
        let path = match model {
            W3cDataModelVersion::V1 => vec!["$.vc.type".to_string()],
            W3cDataModelVersion::V2 => vec!["$.type".to_string()],
        };
        let supported_alg: Vec<String> = Alg::supported()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        InputDescriptor {
            id: vc_type.to_string(),
            format: InputDescriptorFormat {
                jwt_vc_json: InputDescriptorFormatJWTJson { alg: supported_alg },
            },
            constraints: InputDescriptorConstraints {
                fields: vec![InputDescriptorConstraintsFields {
                    path,
                    filter: InputDescriptorConstraintsFieldsFilter {
                        r#type: "string".to_string(),
                        pattern: vc_type.to_string(),
                    },
                }],
            },
        }
    }
}
