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

//! Which identity provider authenticates the users of an app (the `oauth` block of its config).

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::errors::{Errors, Outcome};
use crate::http::USER_TOKEN_HEADER;
use crate::services::token_validator::{
    FixedUserValidator, OauthTokenValidatorTrait, ProxiedTokenValidator, VerifiedTokenValidator,
};
use crate::types::oauth::UserInfo;

/// Who authenticates the users, chosen in the yaml with `provider`. Without the block, `static`
/// as the system user, so nothing has to be configured locally.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum OauthConfig {
    /// The agent's own OAuth server (ds-agent). Not operational yet: nothing starts with it.
    BuiltIn,
    /// Keycloak behind an authenticating proxy, which forwards the user's token.
    Keycloak(KeycloakConfig),
    /// Every request acts as this user. Local development only.
    Static(UserInfo),
}

impl Default for OauthConfig {
    fn default() -> Self {
        Self::Static(UserInfo::system())
    }
}

/// How the user's token reaches the app from the proxy in front of Keycloak.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeycloakConfig {
    /// Header where the proxy forwards the user's token. `Authorization` stays for the
    /// protocols (GNAP, DSP, OID4VCI).
    #[serde(default = "KeycloakConfig::default_user_header")]
    pub user_header: String,
    /// Verify the token (signature, expiry and issuer) instead of trusting the proxy.
    #[serde(default)]
    pub strict: bool,
    /// The realm that issues the tokens (its URL, as in their `iss`); required with `strict`.
    /// The signing keys are fetched from its OpenID configuration.
    #[serde(default)]
    pub issuer: Option<String>,
    /// With `strict`, the client the token has to be for (`aud`); any when not set.
    #[serde(default)]
    pub audience: Option<String>,
}

impl KeycloakConfig {
    fn default_user_header() -> String {
        USER_TOKEN_HEADER.to_string()
    }
}

impl Default for KeycloakConfig {
    fn default() -> Self {
        Self {
            user_header: Self::default_user_header(),
            strict: false,
            issuer: None,
            audience: None,
        }
    }
}

impl OauthConfig {
    /// The validator for this block. Fails with `static` in production (it would let anyone in
    /// as its fixed user), with `strict` and no issuer, and with `built_in`, not operational.
    pub fn validator(&self, is_prod: bool) -> Outcome<Arc<dyn OauthTokenValidatorTrait>> {
        match self {
            OauthConfig::BuiltIn => Err(Errors::security(
                "the built_in OAuth provider is not operational yet; use keycloak or static",
                None,
            )),
            OauthConfig::Keycloak(keycloak) if keycloak.strict => {
                let issuer = keycloak.issuer.clone().ok_or_else(|| {
                    Errors::security("oauth: strict keycloak needs the issuer of the realm", None)
                })?;
                Ok(Arc::new(VerifiedTokenValidator::new(
                    &keycloak.user_header,
                    issuer,
                    keycloak.audience.clone(),
                )))
            }
            OauthConfig::Keycloak(keycloak) => {
                Ok(Arc::new(ProxiedTokenValidator::new(&keycloak.user_header)))
            }
            OauthConfig::Static(_) if is_prod => Err(Errors::security(
                "the static identity provider cannot run in production",
                None,
            )),
            OauthConfig::Static(user) => Ok(Arc::new(FixedUserValidator::new(user.clone()))),
        }
    }
}
