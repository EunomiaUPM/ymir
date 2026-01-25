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

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::errors::{ErrorLogTrait, Errors};
use crate::types::vcs::W3cDataModelVersion;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StuffToIssue {
    pub vc_model: VcModel,
    pub w3c_data_model: Option<W3cDataModelVersion>,
    pub dataspace_id: Option<String>,
    pub federated_catalog_uri: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum VcModel {
    JwtVc,
    SdJwtVc,
}

impl Display for VcModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VcModel::JwtVc => "jwt_vc".to_string(),
            VcModel::SdJwtVc => "sd_jwt_vc".to_string(),
        };

        write!(f, "{}", s)
    }
}

impl FromStr for VcModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jwt_vc" => Ok(VcModel::JwtVc),
            "sd_jwt_vc" => Ok(VcModel::SdJwtVc),
            _ => {
                let error = Errors::parse_new("Invalid VC format role");
                error!("{}", error.log());
                bail!(error)
            }
        }
    }
}
