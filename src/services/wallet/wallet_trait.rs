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

use crate::capabilities::Did;
use crate::data::entities::wallet::{did, key, vc};
use crate::errors::Outcome;
use crate::types::dids::{DidBuilder, DidDocument};
use crate::types::secrets::PemHelper;
use crate::types::wallet::{Identity, WalletInfo};
use async_trait::async_trait;
use std::sync::{Arc};
use tokio::sync::{RwLock};

/// Wallet abstraction.
///
/// This trait represents the full wallet capability set:
/// identity management, DID/VC storage, and protocol handling (OID4VCI/OID4VP).
#[async_trait]
pub trait WalletTrait: Send + Sync + 'static {
    // ===== CORE WALLET STATE =====================================================================

    /// Links the wallet to its backend or runtime environment.
    async fn link(&self) -> Outcome<()>;

    /// Returns a snapshot of the wallet configuration and state.
    async fn get_wallet(&self) -> Outcome<WalletInfo>;

    /// Returns the currently active DID.
    async fn get_did(&self) -> Outcome<Did>;

    /// Returns the DID Document of the active identity.
    async fn get_did_doc(&self) -> Outcome<DidDocument>;

    /// Returns the wallet identity reference shared across services.
    fn get_identity(&self) -> Arc<RwLock<Identity>>;

    // ===== STORAGE (READ ONLY) ===================================================================

    /// Retrieves a DID by its identifier.
    async fn retrieve_did(&self, id: &str) -> Outcome<did::Model>;

    /// Retrieves the default DID configured in the wallet.
    async fn retrieve_default_did(&self) -> Outcome<did::Model>;

    /// Returns all stored DIDs in the wallet.
    async fn retrieve_all_dids(&self) -> Outcome<Vec<did::Model>>;

    /// Retrieves a cryptographic key by its identifier.
    async fn retrieve_key(&self, id: &str) -> Outcome<key::Model>;

    /// Returns all stored cryptographic keys.
    async fn retrieve_all_keys(&self) -> Outcome<Vec<key::Model>>;

    /// Retrieves a verifiable credential by its identifier.
    async fn retrieve_vc(&self, id: &str) -> Outcome<vc::Model>;

    /// Returns all stored verifiable credentials.
    async fn retrieve_all_vcs(&self) -> Outcome<Vec<vc::Model>>;

    // ===== STORAGE (MUTATIONS) ===================================================================

    /// Registers a new cryptographic key from a PEM representation.
    async fn register_key(
        &self,
        pem_helper: &PemHelper,
        alias: Option<String>,
    ) -> Outcome<key::Model>;

    /// Registers a new DID associated with a set of keys.
    async fn register_did(
        &self,
        did_builder: &DidBuilder,
        keys_id: Vec<String>,
        alias: Option<String>,
    ) -> Outcome<did::Model>;

    /// Stores a verifiable credential in the wallet.
    async fn store_vc(&self, vc: String) -> Outcome<vc::Model>;

    /// Sets the default DID for the wallet.
    async fn set_default_did(&self, did: Did) -> Outcome<did::Model>;

    // ===== DELETE OPERATIONS =====================================================================

    /// Deletes a cryptographic key by its identifier.
    async fn delete_key(&self, id: &str) -> Outcome<()>;

    /// Deletes a DID by its identifier.
    async fn delete_did(&self, id: &str) -> Outcome<()>;

    /// Deletes a verifiable credential by its identifier.
    async fn delete_vc(&self, id: &str) -> Outcome<()>;

    // ===== PROTOCOL HANDLING =====================================================================

    /// Processes an OID4VCI issuance flow from a URI.
    async fn process_oid4vci(&self, uri: &str) -> Outcome<()>;

    /// Processes an OID4VP presentation flow from a URI.
    async fn process_oid4vp(&self, uri: &str) -> Outcome<()>;
}
