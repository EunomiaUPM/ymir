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
use std::str::FromStr;

use crate::data::entities::{issuing, recv_interaction};
use crate::errors::Outcome;
use crate::types::gnap::credential_res::CredentialResponse;
use crate::types::gnap::grant_request::InteractStart;
use crate::types::gnap::grant_response::{
    Continue4GResponse, Interact4GResponse, Subject4GResponse,
};
use crate::types::gnap::{AccessToken, ContinueToken};
use crate::types::vcs::VcType;

#[derive(Serialize, Deserialize, Debug)]
pub struct GrantResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#continue: Option<Continue4GResponse>, /* REQUIRED for continuation calls are allowed for this client
                                                 * instance on this grant request */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<AccessToken>, // REQUIRED if an access token is included
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_response: Option<CredentialResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interact: Option<Interact4GResponse>, // REQUIRED if interaction is needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject4GResponse>, // REQUIRED if subject information is included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl GrantResponse {
    pub fn vc_approved(model: &issuing::Model) -> Outcome<Self> {
        let vc_type = VcType::from_str(&model.vc_type)?;
        Ok(Self {
            r#continue: None,
            access_token: None,
            credential_response: Some(CredentialResponse::new(&model.uri, &vc_type)),
            interact: None,
            subject: None,
            instance_id: None,
            error: None,
        })
    }
    pub fn pending(
        option: &InteractStart,
        model: &recv_interaction::Model,
        uri: Option<&str>,
    ) -> Self {
        Self {
            r#continue: Some(Continue4GResponse {
                uri: model.continue_endpoint.clone(),
                wait: None, // TODO Manage wait time
                access_token: ContinueToken::new(&model.continue_token),
            }),
            access_token: None,
            credential_response: None,
            interact: Some(Interact4GResponse::new(option, &model.as_nonce, uri)),
            subject: None,
            instance_id: Some(model.id.clone()),
            error: None,
        }
    }
    pub fn error(error: String) -> Self {
        Self {
            r#continue: None,
            access_token: None,
            credential_response: None,
            interact: None,
            subject: None,
            instance_id: None,
            error: Some(error),
        }
    }
}
