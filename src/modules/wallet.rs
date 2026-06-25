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

use crate::data::entities::wallet::{did, key, vc};
use crate::errors::Outcome;
use crate::services::HasWallet;
use crate::types::dids::{DidBuilder, DidDocument};
use crate::types::secrets::PemHelper;
use crate::types::wallet::{OidcUri, WalletInfo};
use async_trait::async_trait;

/// Business Orchestration Module for the SSI Decentralized Wallet.
///
/// Serves as a high-level facade exposing unified operations for key management,
/// DID lifecycle registration, credential inventory queries, and standard protocol interactions.
///
/// Automatically implements default structural routing to the underlying [`WalletTrait`] implementation.
#[async_trait]
pub trait WalletModuleTrait: HasWallet + Send + Sync + 'static {
    // ===== LIFECYCLE & LINKING ===================================================================

    /// Triggers an out-of-band linkage routine to anchor the wallet inside an ecosystem data space.
    async fn link(&self) -> Outcome<()> {
        self.wallet().link().await
    }

    /// Asserts whether the wallet has been successfully linked and possesses an active identity context.
    async fn is_linked(&self) -> bool {
        self.wallet().get_did().await.is_ok()
    }

    /// Resolves and returns the fully compliant local Decentralized Identifier (DID) Document.
    async fn get_did_doc(&self) -> Outcome<DidDocument> {
        self.wallet().get_did_doc().await
    }

    // ===== IDENTITY PROVISIONING & CRYPTOGRAPHY ==================================================

    /// Registers a new asymmetric keypair registry within the database from a secure private PEM payload.
    async fn register_key(
        &self,
        pem_helper: PemHelper,
        alias: Option<String>,
    ) -> Outcome<key::Model> {
        self.wallet().register_key(&pem_helper, alias).await
    }

    /// Establishes and activates a new local DID binding it to a set of pre-registered verification keys.
    async fn register_did(
        &self,
        did_builder: DidBuilder,
        keys_id: Vec<String>,
        alias: Option<String>,
    ) -> Outcome<did::Model> {
        self.wallet()
            .register_did(&did_builder, keys_id, alias)
            .await
    }

    // ===== RESOURCE PURGING / DELETIONS ==========================================================

    /// Permanently removes a key reference from the database ledger.
    async fn delete_key(&self, id: &str) -> Outcome<()> {
        self.wallet().delete_key(id).await
    }

    /// Permanently removes a local DID registry from the database ledger.
    async fn delete_did(&self, id: &str) -> Outcome<()> {
        self.wallet().delete_did(id).await
    }

    /// Permanently purges an issued Verifiable Credential from the wallet storage.
    async fn delete_credential(&self, id: &str) -> Outcome<()> {
        self.wallet().delete_vc(id).await
    }

    // ===== PROTOCOL INBOUND INTERACTIONS =========================================================

    /// Processes an inbound OpenID4VCI credential offer URI to claim and store a Verifiable Credential.
    async fn process_oidc4vci(&self, payload: OidcUri) -> Outcome<()> {
        self.wallet().process_oid4vci(&payload.uri).await
    }

    /// Processes an inbound OpenID4VP verifiable presentation request challenge to submit an evaluation response.
    async fn process_oidc4vp(&self, payload: OidcUri) -> Outcome<()> {
        self.wallet().process_oid4vp(&payload.uri).await
    }

    // ===== AUDITING & INVENTORY ==================================================================

    /// Gathers structural diagnostic metrics and settings regarding the host wallet instance.
    async fn get_wallet_info(&self) -> Outcome<WalletInfo> {
        self.wallet().get_wallet().await
    }

    /// Resolves the raw string identifier of the default active identity DID (e.g. `did:key:z6M...`).
    async fn get_wallet_did(&self) -> Outcome<String> {
        Ok(self.wallet().get_did().await?.id().to_string())
    }

    /// Retrieves the entire historical inventory of Verifiable Credentials stored in this wallet.
    async fn get_wallet_credentials(&self) -> Outcome<Vec<vc::Model>> {
        self.wallet().retrieve_all_vcs().await
    }
}
