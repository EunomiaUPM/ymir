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

use crate::data::entities::wallet::did::Model;
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;
use async_trait::async_trait;

/// Data Repository Contract for Decentralized Identifier (DID) records lifecycle.
///
/// Inherits foundational CRUD actions from [`CrudRepoTrait`] and encapsulates 
/// identity-switching mechanisms and indexed DID resolution queries.
#[async_trait]
pub trait DidRepoTrait: CrudRepoTrait<Model, Model> + Send + Sync + 'static {

    /// Resolves local metadata for a complete, fully-qualified DID string.
    ///
    /// # Errors
    /// Returns a missing resource error if the exact DID string is not provisioned in the ledger.
    async fn get_by_did(&self, did: &str) -> Outcome<Model>;

    /// Retrieves the active or primary DID designated as the default choice for outbound operations.
    ///
    /// This identity is typically utilized for implicit client handshakes or automated signing 
    /// contexts when no explicit identity is requested by the transaction.
    async fn get_default(&self) -> Outcome<Model>;

    /// Designates a specific DID record as the default primary identity for the execution context.
    ///
    /// Implementations should guarantee that this operation updates transactional state cleanly, 
    /// clearing any previous default flag atomically if multi-tenant constraints apply.
    async fn set_default(&self, id: &str) -> Outcome<Model>;
}