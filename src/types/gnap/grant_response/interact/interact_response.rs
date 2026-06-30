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

use super::UserCodeUri;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InteractResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oid4vp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_code_uri: Option<UserCodeUri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
}

impl InteractResponse {
    // pub fn new(params: InteractResponseParams) -> Self {
    //     let oidc4vp = match params.start {
    //         InteractStart::Oid4VP => params.oid4vp_uri,
    //         _ => None,
    //     };
    //
    //     Self {
    //         oidc4vp,
    //         redirect: None,
    //         app: None,
    //         user_code: None,
    //         user_code_uri: None,
    //         finish: Some(params.callback_nonce),
    //         expires_in: None,
    //     }
    // }
}
