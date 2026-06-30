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

use crate::services::identity::IdentityTrait;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::capabilities::Did;
use crate::data::entities::wallet::did::Model;
use crate::errors::Outcome;
use crate::types::wallet::Identity;

/// Concrete implementation of the operational dynamic identity supervisor.
///
/// Wraps an asynchronous Read-Write lock inside a reference-counted allocation,
/// managing state transitions for the hot-swappable active cryptographic context.
pub struct IdentityManager {
    identity: Arc<RwLock<Option<Identity>>>,
}

impl IdentityManager {
    /// Instantiates an empty identity supervisor initialized to `None`.
    ///
    /// Used during initial node bootstrapping before any base identity has been explicitly asserted.
    pub fn new() -> Self {
        let identity = Arc::new(RwLock::new(None));
        Self { identity }
    }

    /// Bootstraps an identity supervisor by parsing and preloading a database DID model record.
    ///
    /// # Errors
    /// Returns an error if the underlying string representation of the DID fails structural validation.
    pub fn load(model: &Model) -> Outcome<Self> {
        let identity = create_identity(model)?;
        let identity = Arc::new(RwLock::new(Some(identity)));
        Ok(Self { identity })
    }
}

#[async_trait]
impl IdentityTrait for IdentityManager {
    /// Serializes a database model snapshot and updates the active runtime state under an exclusive write lock.
    async fn save_identity(&self, did_model: &Model) -> Outcome<()> {
        let identity = create_identity(did_model)?;
        let mut lock = self.identity.write().await;
        *lock = Some(identity);
        Ok(())
    }

    /// Evicts the operational identity context under an exclusive write lock.
    async fn clear_identity(&self) -> Outcome<()> {
        let mut lock = self.identity.write().await;
        *lock = None;
        Ok(())
    }

    /// Clones the internal pointer allocation to expose the underlying synchronization boundary.
    fn get_identity(&self) -> Arc<RwLock<Option<Identity>>> {
        self.identity.clone()
    }
}

/// Helper constructor to structurally parse database models into structured hot runtime memories.
///
/// Avoids lock contention by executing cryptographic URI parsing and allocation cloning
/// before acquiring exclusive asynchronous write primitives.
fn create_identity(did_model: &Model) -> Outcome<Identity> {
    let did = Did::parse(&did_model.did)?;
    Ok(Identity::new(
        did,
        did_model.did_document.clone(),
        did_model.default_key.clone(),
    ))
}