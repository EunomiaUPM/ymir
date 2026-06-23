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

use std::sync::Arc;

use crate::errors::Outcome;
use crate::types::dids::{DidBuilder, DidDocument};
use crate::types::secrets::PemHelper;
use crate::types::wallet::WalletInfo;
use crate::types::wallet::waltid::{IsLinked, OidcUri};
use crate::data::entities::wallet::{did, vc, key};
use async_trait::async_trait;
use crate::services::HasWallet;
use crate::services::repo::traits::shared::ParticipantRepoTrait;

#[async_trait]
pub trait WalletModuleTrait: HasWallet + Send + Sync + 'static {
    fn participant(&self) -> Arc<dyn ParticipantRepoTrait>;
    async fn link(&self) -> Outcome<()> {
        self.wallet().link().await
    }

    async fn is_linked(&self) -> IsLinked {
        IsLinked::new(self.wallet().get_did().is_ok())
    }

    async fn get_did_doc(&self) -> Outcome<DidDocument> {
        self.wallet().get_did_doc()
    }

    async fn register_key(
        &self,
        pem_helper: PemHelper,
        alias: Option<String>,
    ) -> Outcome<key::Model> {
        self.wallet().register_key(&pem_helper, alias).await
    }

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

    async fn delete_key(&self, id: &str) -> Outcome<()> {
        self.wallet().delete_key(id).await
    }

    async fn delete_did(&self, id: &str) -> Outcome<()> {
        self.wallet().delete_did(id).await
    }

    async fn delete_credential(&self, id: &str) -> Outcome<()> {
        self.wallet().delete_vc(id).await
    }

    async fn process_oidc4vci(&self, payload: OidcUri) -> Outcome<()> {
        self.wallet().process_oid4vci(&payload.uri).await
    }

    async fn process_oidc4vp(&self, payload: OidcUri) -> Outcome<()> {
        self.wallet().process_oid4vp(&payload.uri).await
    }

    async fn get_wallet_info(&self) -> Outcome<WalletInfo> {
        self.wallet().get_wallet().await
    }

    async fn get_wallet_did(&self) -> Outcome<String> {
        Ok(self.wallet().get_did()?.id().to_string())
    }

    async fn get_wallet_credentials(&self) -> Outcome<Vec<vc::Model>> {
        self.wallet().retrieve_all_vcs().await
    }
}
