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

use async_trait::async_trait;
use crate::errors::Outcome;
use crate::services::repo::subtraits::{MatesTrait, MinionsTrait};
use crate::services::wallet::WalletTrait;
use crate::types::dids::{DidBuilder, DidDocument};
use crate::types::secrets::PemHelper;
use crate::types::wallet::WalletInfo;
use crate::types::wallet::{DidModel, KeyModel, VcModel};
use crate::types::wallet::waltid::{IsLinked, OidcUri};

#[async_trait]
pub trait CoreWalletTrait: Send + Sync + 'static {
    fn wallet(&self) -> Arc<dyn WalletTrait>;
    fn mate(&self) -> Option<Arc<dyn MatesTrait>>;
    fn minion(&self) -> Option<Arc<dyn MinionsTrait>>;

    async fn link(&self) -> Outcome<()> {
        let (mate, minion) = self.wallet().link().await?;
        if let Some(mater) = self.mate() {
            mater.force_create(mate).await?;
        }
        if let Some(gru) = self.minion() {
            gru.force_create(minion).await?;
        }
        Ok(())
    }

    async fn is_linked(&self) -> IsLinked {
        IsLinked::new(self.wallet().get_did().is_ok())
    }

    async fn get_did_doc(&self) -> Outcome<DidDocument> {
        self.wallet().get_did_doc()
    }

    async fn register_key(&self, pem_helper: PemHelper, alias: Option<String>) -> Outcome<KeyModel> {
        self.wallet().register_key(&pem_helper, alias).await
    }

    async fn register_did(&self, did_builder: DidBuilder, keys_id: Vec<String>, alias: Option<String>) -> Outcome<DidModel> {
        self.wallet().register_did(&did_builder, keys_id, alias).await
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

    async fn get_wallet_credentials(&self) -> Outcome<Vec<VcModel>> {
        self.wallet().retrieve_all_vcs().await
    }
}
