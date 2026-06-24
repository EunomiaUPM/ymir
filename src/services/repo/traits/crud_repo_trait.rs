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

use crate::errors::Outcome;
use async_trait::async_trait;

/// Unified Data Access Object (DAO) core contract.
///
/// Decouples business services from storage implementations. Operates over
/// a read-only immutable Domain Model (`M`) and a transactional creation payload or Plan (`P`).
#[async_trait]
pub trait CrudRepoTrait<M, P>: Send + Sync + 'static
where
    M: Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    /// Retrieves a paginated subset of domain models.
    /// Default thresholds are determined by the underlying engine.
    async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> Outcome<Vec<M>>;

    /// Fetches a unique domain identity. Returns a missing resource error if not found.
    async fn get_by_id(&self, id: &str) -> Outcome<M>;

    /// Persists a lifecycle initialization plan, transforming it into a complete domain model.
    async fn create(&self, plan: P) -> Outcome<M>;

    /// Synchronizes mutations from an active model back into the storage layer.
    async fn update(&self, model: M) -> Outcome<M>;

    /// Purges a resource record permanently by its unique identifier.
    async fn delete(&self, id: &str) -> Outcome<()>;
}
