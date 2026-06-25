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

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::entities::wallet::did;
use crate::errors::Outcome;
use crate::types::wallet::Identity;

/// Core interface for orchestrating the active decentralized identity loaded in memory.
///
/// Provides thread-safe, non-blocking asynchronous routines to serialize database structures
/// into dynamic runtime configurations, synchronization boundaries, and hot-swappable key material.
#[async_trait]
pub trait IdentityTrait: Send + Sync + 'static {
    /// Commits a database DID model snapshot into the active dynamic runtime state.
    ///
    /// Parses public metadata, structured DID documents, and verification fragments
    /// to initialize or replace the hot-swappable signing context.
    async fn save_identity(&self, did_model: &did::Model) -> Outcome<()>;

    /// Clears the operational system identity from memory, transitioning the node into an unconfigured state.
    ///
    /// Evicts current active cryptographic contexts and prevents unauthorized verification
    /// or outbound credential signature operations.
    async fn clear_identity(&self) -> Outcome<()>;

    /// Yields direct access to the thread-safe asynchronous concurrency boundary.
    ///
    /// Returns a shared pointer wrapping an optimal, architecture-level `RwLock`
    /// for high-concurrency read operations and isolated write synchronization.
    fn get_identity(&self) -> Arc<RwLock<Option<Identity>>>;
}