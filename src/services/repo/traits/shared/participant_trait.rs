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

use crate::data::entities::shared::participant::{Model, Plan};
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;
use crate::types::listing::{ListPage, Listed, ParticipantListFilter, ParticipantSort};
use crate::types::oauth::UserInfo;
use async_trait::async_trait;

/// Peers known to the whole organisation, one row per `participant_id`.
///
/// The CRUD and [`get_batch`](Self::get_batch) read every participant, for system flows (a peer
/// verified by the gatekeeper, refreshing its last interaction). Anything done for a user goes
/// through the `*_visible` methods or [`find_page`](Self::find_page), which apply the visibility
/// rules: a participant is visible if (a) the user added it, (b) someone added it as not
/// `Private`, (c) someone under a role below the user's added it, or (d) it has a received grant
/// under the user's role or one below it, or a public one.
#[async_trait]
pub trait ParticipantRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static {
    /// Every participant found among `ids`, whoever can see it.
    async fn get_batch(&self, ids: &[String]) -> Outcome<Vec<Model>>;
    /// The participant `id` if `user` sees it; missing-resource error otherwise.
    async fn get_visible(&self, user: &UserInfo, id: &str) -> Outcome<Model>;
    /// The participants found among `ids` that `user` sees.
    async fn get_visible_batch(&self, user: &UserInfo, ids: &[String]) -> Outcome<Vec<Model>>;
    /// Lists one page of the participants `filter.tenant` sees, filtered and paged in the
    /// database.
    async fn find_page(
        &self,
        filter: &ParticipantListFilter,
        page: &ListPage<ParticipantSort>,
    ) -> Outcome<Listed<Model>>;
    /// Inserts the participant, or refreshes its nick, base URL and last interaction if it
    /// already exists.
    async fn force_update(&self, plan: Plan) -> Outcome<Model>;
    /// Inserts the participant if it does not exist yet; an existing one is returned untouched,
    /// since it is shared by the whole organisation.
    async fn create_if_absent(&self, plan: Plan) -> Outcome<Model>;
}

#[cfg(feature = "mock")]
mockall::mock! {
    pub ParticipantRepoTrait {}

    #[async_trait]
    impl CrudRepoTrait<Model, Plan> for ParticipantRepoTrait {
        async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> Outcome<Vec<Model>>;
        async fn get_by_id(&self, id: &str) -> Outcome<Model>;
        async fn create(&self, plan: Plan) -> Outcome<Model>;
        async fn update(&self, model: Model) -> Outcome<Model>;
        async fn delete(&self, id: &str) -> Outcome<()>;
    }

    #[async_trait]
    impl ParticipantRepoTrait for ParticipantRepoTrait {
        async fn get_batch(&self, ids: &[String]) -> Outcome<Vec<Model>>;
        async fn get_visible(&self, user: &UserInfo, id: &str) -> Outcome<Model>;
        async fn get_visible_batch(&self, user: &UserInfo, ids: &[String]) -> Outcome<Vec<Model>>;
        async fn find_page(&self, filter: &ParticipantListFilter, page: &ListPage<ParticipantSort>) -> Outcome<Listed<Model>>;
        async fn force_update(&self, plan: Plan) -> Outcome<Model>;
        async fn create_if_absent(&self, plan: Plan) -> Outcome<Model>;
    }
}
