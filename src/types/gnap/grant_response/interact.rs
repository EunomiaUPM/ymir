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

use crate::types::gnap::grant_request::InteractStart;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interact4GResponse {
    pub oidc4vp: Option<String>,
    pub redirect: Option<String>, // REQUIRED 4 if redirection
    pub app: Option<String>,      // ...
    pub user_code: Option<String>,
    pub user_code_uri: Option<UserCodeUri4Int>,
    pub finish: Option<String>,
    pub expires_in: Option<u64>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserCodeUri4Int {
    pub code: String,
    pub uri: String
}

impl Interact4GResponse {
    pub fn new(option: &InteractStart, nonce: &str, uri: Option<&str>) -> Self {
        let mut data = Self {
            oidc4vp: None,
            redirect: None,
            app: None,
            user_code: None,
            user_code_uri: None,
            finish: Some(nonce.to_string()),
            expires_in: None
        };

        if let InteractStart::Oidc4VP = option {
            data.oidc4vp = uri.map(|uri| uri.to_string());
        }

        data
    }
}
