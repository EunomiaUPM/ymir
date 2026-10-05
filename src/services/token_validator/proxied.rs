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
use crate::errors::{Errors, Outcome};
use crate::types::oauth::UserInfo;

/// Reads the user from a JWT that an authenticating proxy (oauth2-proxy in front of Keycloak)
/// already verified and forwarded.
///
/// The token is decoded, **not verified**: this is only safe when the agent is reachable
/// through the proxy alone, with no port of its own exposed.
pub struct ProxiedTokenValidator;

#[async_trait]
impl OauthTokenValidatorTrait for ProxiedTokenValidator {
    async fn validate_token<'a>(&self, token: Option<&'a str>) -> Outcome<UserInfo> {
        let token = token.ok_or_else(|| Errors::unauthorized("missing bearer token", None))?;
        UserInfo::from_token(token)
            .map_err(|e| Errors::unauthorized("token carries no valid user", Some(Box::new(e))))
    }
}
