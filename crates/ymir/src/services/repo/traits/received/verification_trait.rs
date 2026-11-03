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

use crate::data::entities::received::verification::{Model, Plan};
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Data Repository Contract for Received OpenID4VP Presentation Requests.
///
/// Tracks verification requests received from external Verifiers, storing session states
/// while the local wallet processes and constructs the target Verifiable Presentation response.
#[async_trait]
pub trait RecvVerificationRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static {
    /// Resolves an active inbound verification session via its OAuth2/OIDC cross-network `state` parameter.
    ///
    /// Essential for securely mapping incoming token/presentation callback handshakes
    /// back to the initial authorization transactional context.
    async fn get_by_state(&self, state: &str) -> Outcome<Model>;

    /// The latest verification of the grant.
    async fn get_latest_by_grant(&self, grant_id: &str) -> Outcome<Model>;

    /// Every verification of the grant, oldest first.
    async fn get_all_by_grant(&self, grant_id: &str) -> Outcome<Vec<Model>>;

    /// Marks as expired the verifications still pending that were created before `created_before`.
    async fn expire_stale_pending(
        &self,
        created_before: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Outcome<u64>;
}

#[cfg(feature = "mock")]
mockall::mock! {
    pub RecvVerificationRepoTrait {}

    #[async_trait]
    impl CrudRepoTrait<Model, Plan> for RecvVerificationRepoTrait {
        async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> Outcome<Vec<Model>>;
        async fn get_by_id(&self, id: &str) -> Outcome<Model>;
        async fn create(&self, plan: Plan) -> Outcome<Model>;
        async fn update(&self, model: Model) -> Outcome<Model>;
        async fn delete(&self, id: &str) -> Outcome<()>;
    }

    #[async_trait]
    impl RecvVerificationRepoTrait for RecvVerificationRepoTrait {
        async fn get_by_state(&self, state: &str) -> Outcome<Model>;
        async fn get_latest_by_grant(&self, grant_id: &str) -> Outcome<Model>;
        async fn get_all_by_grant(&self, grant_id: &str) -> Outcome<Vec<Model>>;
        async fn expire_stale_pending(
            &self,
            created_before: DateTime<Utc>,
            now: DateTime<Utc>,
        ) -> Outcome<u64>;
    }
}
