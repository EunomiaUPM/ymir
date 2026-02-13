/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
use crate::services::repo::subtraits::{MatesTrait, MinionsTrait};
use crate::services::wallet::WalletTrait;
use crate::types::dids::dids_info::DidsInfo;
use crate::types::wallet::{KeyDefinition, OidcUri, WalletCredentials, WalletInfo};
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait CoreWalletTrait: Send + Sync + 'static {
    fn wallet(&self) -> Arc<dyn WalletTrait>;
    fn mate(&self) -> Option<Arc<dyn MatesTrait>>;
    fn minion(&self) -> Option<Arc<dyn MinionsTrait>>;
    async fn register(&self) -> Outcome<()> {
        self.wallet().register().await
    }
    async fn login(&self) -> Outcome<()> {
        self.wallet().login().await
    }
    async fn logout(&self) -> Outcome<()> {
        self.wallet().logout().await
    }
    async fn onboard(&self) -> Outcome<()> {
        let (mate, minion) = self.wallet().onboard().await?;
        if let Some(mater) = self.mate() {
            mater.force_create(mate).await?;
        }

        if let Some(gru) = self.minion() {
            gru.force_create(minion).await?;
        }

        Ok(())
    }
    async fn partial_onboard(&self) -> Outcome<()> {
        self.wallet().partial_onboard().await
    }
    async fn get_did_doc(&self) -> Outcome<Value> {
        self.wallet().get_did_doc().await
    }
    async fn register_key(&self) -> Outcome<()> {
        self.wallet().register_key().await
    }
    async fn register_did(&self) -> Outcome<()> {
        self.wallet().register_did().await
    }
    async fn delete_key(&self, key: KeyDefinition) -> Outcome<()> {
        self.wallet().delete_key(key).await
    }
    async fn delete_did(&self, did_info: DidsInfo) -> Outcome<()> {
        self.wallet().delete_did(did_info).await
    }
    async fn process_oidc4vci(&self, payload: OidcUri) -> Outcome<()> {
        let cred_offer = self.wallet().resolve_credential_offer(&payload).await?;
        let _issuer_metadata = self.wallet().resolve_credential_issuer(&cred_offer).await?;
        self.wallet().use_offer_req(&payload, &cred_offer).await
    }
    async fn process_oidc4vp(&self, payload: OidcUri) -> Outcome<Option<String>> {
        let vpd = self.wallet().get_vpd(&payload).await?;
        let vcs_id = self.wallet().get_matching_vcs(&vpd).await?;
        self.wallet().present_vp(&payload, vcs_id).await
    }
    async fn get_wallet_info(&self) -> Outcome<WalletInfo> {
        self.wallet().get_wallet().await
    }
    async fn get_wallet_did(&self) -> Outcome<String> {
        self.wallet().get_did().await
    }
    async fn get_wallet_credentials(&self) -> Outcome<Vec<WalletCredentials>> {
        self.wallet().retrieve_wallet_credentials().await
    }
}
