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

use async_trait::async_trait;

use super::OauthTokenValidatorTrait;
use crate::errors::Outcome;
use crate::types::oauth::UserInfo;

/// Authenticates every request as the same user, token or not. For local development only.
pub struct FixedUserValidator {
    user: UserInfo,
}

impl FixedUserValidator {
    pub fn new(user: UserInfo) -> Self {
        Self { user }
    }
}

#[async_trait]
impl OauthTokenValidatorTrait for FixedUserValidator {
    async fn validate_token<'a>(&self, _token: Option<&'a str>) -> Outcome<UserInfo> {
        Ok(self.user.clone())
    }
}
