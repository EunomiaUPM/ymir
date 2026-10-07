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

use crate::data::entities::shared::participant_relation::Model;
use crate::errors::Outcome;
use crate::types::oauth::UserInfo;
use async_trait::async_trait;

/// Which user added which participant, under which role and with what visibility. Keyed by
/// `(user_id, participant_id)`, so it has no `CrudRepoTrait` (single-key ids) nor `Plan`.
#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait ParticipantRelationRepoTrait: Send + Sync + 'static {
    /// The relation of `user_id` with `participant_id`; missing-resource error if none.
    async fn get(&self, user_id: &str, participant_id: &str) -> Outcome<Model>;

    /// Every relation a participant has, whoever added it. For system flows; anything done for
    /// a user goes through [`get_visible_by_participant`](Self::get_visible_by_participant).
    async fn get_by_participant(&self, participant_id: &str) -> Outcome<Vec<Model>>;

    /// The relations of `participant_id` that `user` sees: its own and those added under a role
    /// below its own, whole; those of anyone else unless `Private`, and if `Anonymous` without
    /// who added them (`user_id` set to [`ANONYMOUS_USER_ID`](crate::data::entities::shared::participant_relation::ANONYMOUS_USER_ID), no `username`). The root sees
    /// them all, whole.
    async fn get_visible_by_participant(
        &self,
        user: &UserInfo,
        participant_id: &str,
    ) -> Outcome<Vec<Model>>;

    /// Creates the relation, or updates its username, role and visibility if the user already had
    /// one.
    async fn force_update(&self, relation: Model) -> Outcome<Model>;

    async fn delete(&self, user_id: &str, participant_id: &str) -> Outcome<()>;
}
