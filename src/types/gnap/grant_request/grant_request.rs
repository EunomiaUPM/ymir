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
use crate::data::entities::sent::interaction;
use crate::types::vcs::{VcTypeConfig};
use super::access::{AccessRequest, AccessType, ResourceAccess};
use super::client::Client;
use super::grant_request_kind::GrantRequestKind;
use super::interact::{InteractAction, InteractRequest};
use super::subject::SubjectRequest;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrantRequest {
    #[serde(flatten)]
    pub kind: GrantRequestKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<SubjectRequest>,
    pub client: Client,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interact: Option<InteractRequest>,
}

impl GrantRequest {
    pub fn new_vc(client: Client, vc_type: VcTypeConfig, model: &interaction::Model) -> Self {
        let credential_request = AccessRequest {
            access: ResourceAccess {
                r#type: AccessType::VcExchange,
                actions: Some(vec![InteractAction::RequestVc]),
                locations: None,
                datatypes: Some(vec![vc_type.to_string()]),
                identifier: None,
                privileges: None,
            },
            label: None,
            flags: None,
        };

        Self {
            kind: GrantRequestKind::CredentialRequest { credential_request },
            subject: None,
            client,
            user: None,
            interact: Some(InteractRequest::new(model.clone())),
        }
    }

    pub fn new_token(client: Client, actions: Vec<InteractAction>, model: &interaction::Model) -> Self {
        let access_token = AccessRequest {
            access: ResourceAccess {
                r#type: AccessType::ApiAccess,
                actions: Some(actions),
                locations: None,
                datatypes: None,
                identifier: None,
                privileges: None,
            },
            label: None,
            flags: None,
        };

        Self {
            kind: GrantRequestKind::AccessToken { access_token },
            subject: None,
            client,
            user: None,
            interact: Some(InteractRequest::new(model.clone())),
        }
    }
}
