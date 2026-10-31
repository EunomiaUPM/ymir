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

//! Everything OAuth in one namespace: the user ([`UserInfo`], its role path and the access rule
//! over the role tree), the token validators, the [`OauthHttpMiddleware`] and the `UserInfo`
//! extractor, record ownership ([`Owner`], [`OwnerScope`]) and, with `grpc`, [`GrpcAuth`].
//!
//! | `oauth.provider` | Validator ([`OauthConfig::validator`]) | Token |
//! |---|---|---|
//! | `keycloak` | decodes the JWT that oauth2-proxy verified and forwarded | required |
//! | `static` (default) | every request is the user in the config; refused in production | ignored |
//! | `built_in` | not operational yet: the agent refuses to start | — |
//!
//! ## 1. The rule
//!
//! Roles are paths hanging from the root `/admin` (`/admin/upm/dit`); the second segment is the
//! company. A user reaches (reads and writes) its own records and those created under a role
//! below its own, never those of a user sharing its role; `/admin` reaches everything. See
//! [`UserInfo::reaches`].
//!
//! ## 2. Protecting an HTTP router
//!
//! Put [`OauthHttpMiddleware::run`] as a route layer with the validator as state. It stores the
//! user in the request; handlers take it as an extractor.
//!
//! ```rust,ignore
//! pub(crate) fn router(self) -> Router {
//!     Router::new()
//!         .route("/", get(Self::handle_list))
//!         .with_state(self.clone())
//!         .route_layer(from_fn_with_state(self.validator, OauthHttpMiddleware::run))
//! }
//!
//! async fn handle_list(State(s): State<Self>, user: UserInfo) -> AppResult<Json<Page>> {
//!     Ok(Json(s.service.list(&user).await?))
//! }
//! ```
//!
//! ## 3. Authorizing in the service layer
//!
//! Records keep their [`Owner`] (`user_id`, `user_role`, `visibility`), stamped when created:
//! the user's own, or the role of a peer's grant for what a peer opens. Repositories take an
//! [`OwnerScope`]: reads with [`OwnerScope::seeing`], writes with [`OwnerScope::acting`], and
//! in-process flows that already hold the record's id with [`OwnerScope::All`]. A record out of
//! scope looks like a missing one (404). See [`ownership`].
//!
//! ## 4. gRPC
//!
//! [`GrpcAuth`] does the same from tonic metadata, returning a `Status` on failure.
//!
//! [`GrpcAuth`]: grpc::GrpcAuth
//! [`UserInfo`]: crate::types::oauth::UserInfo
//! [`UserInfo::reaches`]: crate::types::oauth::UserInfo::reaches
//! [`UserInfo::system`]: crate::types::oauth::UserInfo::system
//! [`OauthTokenValidatorTrait`]: crate::services::token_validator::OauthTokenValidatorTrait
//! [`OauthHttpMiddleware`]: crate::http::OauthHttpMiddleware
//! [`OauthHttpMiddleware::run`]: crate::http::OauthHttpMiddleware::run
//!
//! ## 5. In-process calls
//!
//! A local facade or a seeder has no request behind it: it acts as [`UserInfo::system`], the
//! root.

#[cfg(feature = "grpc")]
pub mod grpc;
pub mod ownership;

pub use ownership::{OwnedTrait, Owner, OwnerScope, OwnershipTrait, acting_as};

pub use crate::config::types::OauthConfig;
pub use crate::http::{OauthHttpMiddleware, USER_TOKEN_HEADER};
pub use crate::services::token_validator::{
    FixedUserValidator, OauthTokenValidatorTrait, ProxiedTokenValidator, VerifiedTokenValidator,
};
pub use crate::types::oauth::{RolePath, RoleTrait, SYSTEM_USER_ID, UserInfo, UserTrait};

/// Header / metadata key carrying the bearer token.
pub const AUTHORIZATION_HEADER: &str = "authorization";

#[cfg(test)]
mod middleware_tests;
#[cfg(test)]
mod tests;
