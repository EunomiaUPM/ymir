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
use crate::types::participants::ParticipantType;
use async_trait::async_trait;

/// Data Repository Contract for Participant Domain Management.
///
/// Extends the foundational [`CrudRepoTrait`] over Postgres tables to execute specific
/// domain spaces queries, token authorization validations, and atomic batch fetches.
#[async_trait]
pub trait ParticipantRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static {
    /// Resolves the operational identity context of the execution host ("me").
    async fn get_me(&self) -> Outcome<Model>;

    /// Queries multi-tenant environments filtering participants by their role type.
    async fn filter_by_type(&self, participant_type: ParticipantType) -> Outcome<Vec<Model>>;

    /// Locates an active participant bound to a specific API bearer or authorization token.
    async fn get_by_token(&self, token: &str) -> Outcome<Model>;

    /// Optimized vectorized query to retrieve multiple records simultaneously, reducing DB roundtrips.
    async fn get_batch(&self, ids: &[String]) -> Outcome<Vec<Model>>;

    /// Performs an upsert-style force update bypassing standard transaction mutation checks.
    async fn force_update(&self, plan: Plan) -> Outcome<Model>;
}
