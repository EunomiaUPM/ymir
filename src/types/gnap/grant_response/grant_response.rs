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

use crate::data::entities::recv_interaction;
use crate::types::gnap::grant_response::{
    Continue4GResponse, Interact4GResponse, Subject4GResponse,
};
use crate::types::gnap::{AccessToken, GRMethod};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GrantResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#continue: Option<Continue4GResponse>, /* REQUIRED for continuation calls are allowed for this client
                                                 * instance on this grant request */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<AccessToken>, // REQUIRED if an access token is included
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interact: Option<Interact4GResponse>, // REQUIRED if interaction is needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject4GResponse>, // REQUIRED if subject information is included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>, // TODO
}

impl GrantResponse {
    pub fn new(option: GRMethod, model: &recv_interaction::Model, uri: Option<String>) -> Self {
        Self {
            r#continue: Some(Continue4GResponse {
                uri: model.continue_endpoint.clone(),
                wait: None, // TODO Manage wait time
                access_token: AccessToken::default(model.continue_token.clone()),
            }),
            access_token: None,
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
            interact: None,
            subject: None,
            instance_id: None,
            error: Some(error),
        }
    }
}
