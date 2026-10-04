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
use crate::types::participants::Visibility;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::utils::{OneOrMany, decode_url_safe_no_pad};

/// User id of the agent itself when it acts in-process, with no request behind it.
pub const SYSTEM_USER_ID: &str = "system";

/// Who is acting: the user, the role it acts under and the rest of its profile. Travels as one
/// value so a user id is never paired with someone else's role.
///
/// It also carries the access rules over the role tree:
/// - a user **reaches** a record with an author (its own, or created under a role hanging below
///   its own; users sharing a role do not reach each other's) and **handles** a record with only
///   a role (the same role or one below it). The root [`RolePath::root`] reaches and handles
///   everything. Reaching or handling lets the user read **and act on** the record;
/// - a user **sees** a record it reaches or handles, or one whose [`Visibility`] opens it to
///   everyone. Seeing only lets it read;
/// - a user **manages** (creates, changes, deletes) only identities whose role hangs below its
///   own; the root manages all of them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    user_id: String,
    email: Option<String>,
    role: RolePath,
    /// Every other claim the token carried, about the user or about the token itself.
    #[serde(flatten)]
    extra: Map<String, Value>,
}

/// Two values are the same user acting under the same role; `extra` is left out, since it
/// also holds claims of the particular token (expiry, id, session…).
impl PartialEq for UserInfo {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id && self.email == other.email && self.role == other.role
    }
}

impl Eq for UserInfo {}

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
    /// A user built by whoever issues its tokens; `extra` holds any other claim about it.
    pub fn new(
        user_id: impl Into<String>,
        email: Option<String>,
        role: RolePath,
        extra: Map<String, Value>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            email,
            role,
            extra,
        }
    }

    /// Reads the user from the payload of a compact JWT (`header.payload.signature`): `sub`,
    /// `email`, `role`, and every other claim into `extra`. `role` may be a single path or a list (e.g.
    /// Keycloak groups); with several, the highest one (shortest path) is taken.
    ///
    /// Only the payload is read: the header and signature are **not** checked, so the token
    /// must come already verified (by the proxy in front, or by whoever issued it). Fails if the
    /// token is not a JWT, if `sub` or `role` is missing or a role is not a valid path.
    pub fn from_token(token: &str) -> Outcome<Self> {
        let parts: Vec<&str> = token.split('.').collect();
        let [_, payload, _] = parts[..] else {
            return Err(Errors::format(
                BadFormat::Received,
                "token is not a JWT",
                None,
            ));
        };
        let claims: UserClaims = serde_json::from_slice(&decode_url_safe_no_pad(payload)?)
            .map_err(|e| {
                let reason = "token payload is not valid claims";
                Errors::format(BadFormat::Received, reason, Some(Box::new(e)))
            })?;

        let role = claims
            .role
            .iter()
            .map(|role| role.parse::<RolePath>())
            .collect::<Outcome<Vec<_>>>()?
            .into_iter()
            .min_by_key(|role| role.as_str().len())
            .ok_or_else(|| Errors::format(BadFormat::Received, "token carries no role", None))?;

        Ok(Self {
            user_id: claims.sub,
            email: claims.email,
            role,
            extra: claims.extra,
        })
    }

    /// The agent acting on its own behalf (in-process calls, seeders), as the root.
    pub fn system() -> Self {
        Self::new(SYSTEM_USER_ID, None, RolePath::root(), Map::new())
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

    /// The login name (`preferred_username`), when the token carried one.
    pub fn username(&self) -> Option<&str> {
        self.extra.get("preferred_username").and_then(Value::as_str)
    }

    pub fn is_root(&self) -> bool {
        self.role.is_root()
    }

    pub fn require_root(&self) -> Outcome<()> {
        match self.role.is_root() {
            true => Ok(()),
            false => Err(Errors::forbidden("Only admins can perform this action", None)),
        }
    }

    /// Whether the user reaches a record created by `owner_id` under `owner_role`.
    pub fn reaches(&self, owner_id: &str, owner_role: &RolePath) -> bool {
        self.is_root() || owner_id == self.user_id || owner_role.is_below(&self.role)
    }

    /// Fails unless the user reaches the record; an unreachable record looks like a missing one.
    pub fn ensure_reaches(&self, owner_id: &str, owner_role: &RolePath, id: &str) -> Outcome<()> {
        if self.reaches(owner_id, owner_role) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }

//     /// Whether the user may manage an identity holding `role`.
//     pub fn manages(&self, role: &RolePath) -> bool {
//         self.is_root() || role.is_below(&self.role)
//     }

//     /// Fails with a forbidden error unless the user [manages](Self::manages) `role`.
//     pub fn require_manages(&self, role: &RolePath) -> Outcome<()> {
//         if self.manages(role) {
//             Ok(())
//         } else {
//             Err(Errors::forbidden(
//                 format!("forbidden: role '{role}' does not hang below the caller's"),
//                 None,
//             ))
//         }
//     }

    /// Whether the user handles a record assigned to `role`: the same role, one below it, or any
    /// for the root.
    pub fn handles(&self, role: &RolePath) -> bool {
        self.is_root() || *role == self.role || role.is_below(&self.role)
    }

    /// Fails unless the user handles the record; an unhandled record looks like a missing one.
    pub fn ensure_handles(&self, role: &RolePath, id: &str) -> Outcome<()> {
        if self.handles(role) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }

    /// Whether the user sees a record created by `owner_id` under `owner_role`: it reaches it,
    /// or its visibility is not `Private` (an `Anonymous` one is seen without its author).
    pub fn sees(&self, owner_id: &str, owner_role: &RolePath, visibility: &Visibility) -> bool {
        *visibility != Visibility::Private || self.reaches(owner_id, owner_role)
    }

    /// Fails unless the user [sees](Self::sees) the record, which then looks like a missing one.
    pub fn ensure_sees(
        &self,
        owner_id: &str,
        owner_role: &RolePath,
        visibility: &Visibility,
        id: &str,
    ) -> Outcome<()> {
        if self.sees(owner_id, owner_role, visibility) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }

    /// Whether the user sees a record assigned to `role`: it handles it, or it is `Public`.
    pub fn sees_team(&self, role: &RolePath, visibility: &Visibility) -> bool {
        *visibility == Visibility::Public || self.handles(role)
    }

    /// Fails unless the user [sees](Self::sees_team) the record, which then looks like a missing
    /// one.
    pub fn ensure_sees_team(&self, role: &RolePath, visibility: &Visibility, id: &str) -> Outcome<()> {
        if self.sees_team(role, visibility) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }
}
