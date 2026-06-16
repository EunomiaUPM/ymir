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
use crate::data::entities::received::interaction;
use crate::data::entities::shared::resource_req;
use super::credential_response::CredentialResponse;
use super::error_code::ErrorCode;
use super::interact::InteractResponse;
use super::subject::SubjectResponse;
use super::{Continuation};
use crate::types::gnap::access_token::{AccessToken, ContinueToken};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GrantResponse {
    Approved(ApprovedResponse),
    Pending(PendingResponse),
    Processing(ProcessingResponse),
    Error(ErrorResponse),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApprovedResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#continue: Option<Continuation>,
    #[serde(flatten)]
    pub kind: GrantResponseKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<SubjectResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PendingResponse {
    pub r#continue: Continuation,
    pub interact: InteractResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessingResponse {
    pub r#continue: Continuation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub error: ErrorCode,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GrantResponseKind {
    AccessToken { access_token: AccessToken },
    CredentialResponse { credential_response: CredentialResponse },
}

impl GrantResponse {
    pub fn token_approved(token: impl Into<String>, model: &resource_req::Model) -> Self {
        let res = ApprovedResponse {
            r#continue: None,
            kind: GrantResponseKind::AccessToken {
                access_token: AccessToken::new(token, model.clone()),
            },
            subject: None,
            instance_id: None,
        };

        GrantResponse::Approved(res)
    }

    // pub fn vc_approved(params: VcApprovedParams) -> Self {
    //     Self {
    //         r#continue: None,
    //         access_token: None,
    //         credential_response: Some(CredentialResponse {
    //             credential_uri: params.credential_uri,
    //             credential_type: params.vc_type.to_conf(),
    //         }),
    //         interact: None,
    //         subject: None,
    //         instance_id: None,
    //         error: None,
    //     }
    // }
    //
    pub fn pending(uri: impl Into<String>, model: &interaction::Model) -> Self {
        // BY DEFAULT IN THIS USE CASE, VERIFICATION IS DONE THROUGH OID4VC, THAT IS WHY THE REST REMAIN AS NONE
        GrantResponse::Pending(
            PendingResponse {
                r#continue: Continuation {
                    uri: model.continue_endpoint.clone(),
                    wait: None,
                    access_token: ContinueToken::new(model.continue_token.clone()),
                },
                interact: InteractResponse {
                    oid4vp: Some(uri.into()),
                    redirect: None,
                    app: None,
                    user_code: None,
                    user_code_uri: None,
                    finish: Some(model.as_nonce.clone()),
                    expires_in: None,
                },
                instance_id: Some(model.id.clone()),
            }
        )
    }

    pub fn processing(model: &interaction::Model) -> Self {
        GrantResponse::Processing(
            ProcessingResponse {
                r#continue: Continuation {
                    uri: model.continue_endpoint.clone(),
                    wait: None,
                    access_token: ContinueToken::new(model.continue_token.clone()),
                },
                instance_id: Some(model.id.clone()),
            }
        )
    }
    //
    // pub fn error(code: ErrorCode) -> Self {
    //     Self {
    //         r#continue: None,
    //         access_token: None,
    //         credential_response: None,
    //         interact: None,
    //         subject: None,
    //         instance_id: None,
    //         error: Some(code),
    //     }
    // }
}
