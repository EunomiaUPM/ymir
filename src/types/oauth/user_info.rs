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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::RolePath;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::jwt::Jwt;
use crate::utils::OneOrMany;

/// Registered JWT claims that describe the token, not the user; left out of `extra`.
const TOKEN_CLAIMS: [&str; 6] = ["iss", "aud", "exp", "nbf", "iat", "jti"];

/// Who is acting: the user, the role it acts under and the rest of its profile. Travels as one
/// value so a user id is never paired with someone else's role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInfo {
    user_id: String,
    email: Option<String>,
    role: RolePath,
    /// Any other claim about the user.
    #[serde(flatten)]
    extra: Map<String, Value>,
}

/// Claims as they arrive in the token.
#[derive(Deserialize)]
struct UserClaims {
    sub: String,
    email: Option<String>,
    role: OneOrMany<String>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

impl UserInfo {
    /// Reads the user from the claims of `jwt`: `sub`, `email`, `role` and every other user
    /// claim. `role` may be a single path or a list (e.g. Keycloak groups); with several, the
    /// highest one (shortest path) is taken.
    ///
    /// The token is **not** verified here: `jwt` must come already checked (signature,
    /// issuer, audience, expiry). Fails if `sub` or `role` is missing or a role is not a valid
    /// path.
    pub fn from_jwt(jwt: &Jwt) -> Outcome<Self> {
        let mut claims: UserClaims = jwt.unsafe_claims()?;

        let role = claims
            .role
            .iter()
            .map(|role| role.parse::<RolePath>())
            .collect::<Outcome<Vec<_>>>()?
            .into_iter()
            .min_by_key(|role| role.as_str().len())
            .ok_or_else(|| Errors::format(BadFormat::Received, "token carries no role", None))?;

        for claim in TOKEN_CLAIMS {
            claims.extra.remove(claim);
        }

        Ok(Self {
            user_id: claims.sub,
            email: claims.email,
            role,
            extra: claims.extra,
        })
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    pub fn role(&self) -> &RolePath {
        &self.role
    }

    pub fn extra(&self) -> &Map<String, Value> {
        &self.extra
    }
}
