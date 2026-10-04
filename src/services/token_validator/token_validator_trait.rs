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

use crate::errors::Outcome;
use crate::types::oauth::UserInfo;

/// Port each identity provider implements: who is behind a request's bearer token.
#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait TokenValidatorTrait: Send + Sync + 'static {
    /// The user behind `token`, `None` when the request carries none; any failure is a 401.
    async fn validate_token<'a>(&self, token: Option<&'a str>) -> Outcome<UserInfo>;
}
