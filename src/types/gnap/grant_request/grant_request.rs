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

use serde::{Deserialize, Serialize};

use super::AccessTokenRequirements4GR;
use super::Interact4GR;
use super::{Client4GR, InteractActions};
use crate::data::entities::req_interaction;
use crate::types::gnap::grant_request::credential_request_req::CredentialRequest4GR;
use crate::types::gnap::grant_request::subject::Subject4GR;
use crate::types::vcs::VcType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrantRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<AccessTokenRequirements4GR>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_request: Option<CredentialRequest4GR>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject4GR>, // REQUIRED if requesting subject information
    pub client: Client4GR,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    pub interact: Option<Interact4GR>,
}

impl GrantRequest {
    pub fn new_vc(client: &Client4GR, vc_type: &VcType, model: &req_interaction::Model) -> Self {
        Self {
            access_token: None,
            credential_request: Some(CredentialRequest4GR::new(vc_type)),
            subject: None,
            client: client.clone(),
            user: None,
            interact: Some(Interact4GR::new(model)),
        }
    }
    pub fn new_token(
        client: &Client4GR,
        actions: Option<&[InteractActions]>,
        model: &req_interaction::Model,
    ) -> Self {
        Self {
            access_token: Some(AccessTokenRequirements4GR::new(actions)),
            credential_request: None,
            subject: None,
            client: client.clone(),
            user: None,
            interact: Some(Interact4GR::new(model)),
        }
    }
}
