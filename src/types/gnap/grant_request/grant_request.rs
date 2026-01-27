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

use serde::{Deserialize, Serialize};

use super::Client4GR;
use super::Interact4GR;
use super::{AccessTokenRequirements4GR, TokenReqTypeGR};
use crate::data::entities::req_interaction;
use crate::types::gnap::GRUse;
use crate::types::gnap::grant_request::subject::Subject4GR;
use crate::types::vcs::VcType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrantRequest {
    pub access_token: AccessTokenRequirements4GR,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject4GR>, // REQUIRED if requesting subject information
    pub client: Client4GR,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    pub interact: Option<Interact4GR>
}

impl GrantRequest {
    pub fn new(
        option: GRUse,
        client: Client4GR,
        vc_type: Option<VcType>,
        model: &req_interaction::Model
    ) -> Self {
        Self {
            access_token: AccessTokenRequirements4GR::new(
                option,
                vc_type,
                Some(TokenReqTypeGR::Bearer)
            ),
            subject: None,
            client,
            user: None,
            interact: Some(Interact4GR::new(model))
        }
    }
}
