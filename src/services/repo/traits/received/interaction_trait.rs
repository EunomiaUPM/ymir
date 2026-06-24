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

use async_trait::async_trait;

use crate::data::entities::received::interaction::{Model, Plan};
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;

/// Data Repository Contract for Inbound GNAP User Interaction sessions.
///
/// Tracks the orchestration of authorization screens or interaction flows requested
/// by external entities that must be resolved by the local platform's users.
#[async_trait]
pub trait RecvInteractionRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync {
    /// Locates an interaction record using its unique GNAP Continuation Identifier (`cont_id`).
    ///
    /// Executed when a client returns to the continuation endpoint to claim tokens
    /// after the out-of-band user interaction has finalized successfully.
    async fn get_by_cont_id(&self, cont_id: &str) -> Outcome<Model>;
}