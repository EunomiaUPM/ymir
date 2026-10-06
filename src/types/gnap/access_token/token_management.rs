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

use crate::types::gnap::access_token::BoundToken;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenManagement {
    pub uri: String,
    pub access_token: BoundToken,
}

impl TokenManagement {
    pub fn new(uri: impl Into<String>, access_token: BoundToken) -> Self {
        Self {
            uri: uri.into(),
            access_token,
        }
    }
}
