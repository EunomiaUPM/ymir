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

use crate::data::entities::shared::issuance::{Model, Plan};
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;
use async_trait::async_trait;

/// Data Repository Contract for OpenID4VCI v1.0 Issuance Sessions.
///
/// Orchestrates the persistence of active credential issuance states, tracking 
/// transactional lifetimes from initialization (`Plan`) down to ephemeral token authorization evaluations (`Model`).
#[async_trait]
pub trait IssuanceRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static {

    /// Retrieves an issuance session using its associated pre-authorized code.
    ///
    /// Utilized during the Pre-Authorized Code Flow handshake when a remote Wallet 
    /// hits the token endpoint using a code extracted from a `Credential Offer`.
    ///
    /// # Errors
    /// Returns a missing resource error if the pre-auth code is invalid or expired.
    async fn get_by_pre_auth_code(&self, code: &str) -> Outcome<Model>;

    /// Locates an active issuance transaction bound to a specific OAuth 2.0 / GNAP Access Token.
    ///
    /// Executed at the `/credential` endpoint to guarantee that the incoming request 
    /// possesses authorized coverage over the requested Verifiable Credentials configuration layout.
    async fn get_by_token(&self, token: &str) -> Outcome<Model>;
}