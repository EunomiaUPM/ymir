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

use async_trait::async_trait;
use reqwest::Response;
use serde_json::Value;

use crate::data::entities::{mates, minions};
use crate::errors::Outcome;
use crate::types::dids::dids_info::DidsInfo;
use crate::types::wallet::{
    CredentialOfferResponse, KeyDefinition, MatchingVCs, OidcUri, Vpd, WalletCredentials,
    WalletInfo, WalletSession
};

#[async_trait]
pub trait WalletTrait: Send + Sync + 'static {
    // BASIC
    async fn register(&self) -> Outcome<()>;
    async fn login(&self) -> Outcome<()>;
    async fn logout(&self) -> Outcome<()>;
    async fn onboard(&self) -> Outcome<(mates::NewModel, minions::NewModel)>;
    async fn partial_onboard(&self) -> Outcome<()>;
    // GET FROM MANAGER (It gives a cloned Value, not a reference)
    async fn get_wallet(&self) -> Outcome<WalletInfo>;
    async fn first_wallet_mut(&self) -> Outcome<tokio::sync::MutexGuard<'_, WalletSession>>;
    async fn get_did(&self) -> Outcome<String>;
    async fn get_token(&self) -> Outcome<String>;
    async fn get_did_doc(&self) -> Outcome<Value>;
    async fn get_key(&self) -> Outcome<KeyDefinition>;
    // RETRIEVE FROM WALLET
    async fn retrieve_wallet_info(&self) -> Outcome<()>;
    async fn retrieve_wallet_keys(&self) -> Outcome<()>;
    async fn retrieve_wallet_dids(&self) -> Outcome<()>;
    async fn retrieve_wallet_credentials(&self) -> Outcome<Vec<WalletCredentials>>;
    // REGISTER STUFF IN WALLET
    async fn register_key(&self) -> Outcome<()>;
    async fn register_did(&self) -> Outcome<()>;
    async fn reg_did_jwk(&self) -> Outcome<Response>;
    async fn reg_did_web(&self) -> Outcome<Response>;
    async fn set_default_did(&self) -> Outcome<()>;
    // DELETE STUFF FROM WALLET
    async fn delete_key(&self, key: KeyDefinition) -> Outcome<()>;
    async fn delete_did(&self, did_info: DidsInfo) -> Outcome<()>;
    async fn resolve_credential_offer(&self, payload: &OidcUri)
    -> Outcome<CredentialOfferResponse>;
    async fn resolve_credential_issuer(
        &self,
        cred_offer: &CredentialOfferResponse
    ) -> Outcome<Value>;
    async fn use_offer_req(
        &self,
        payload: &OidcUri,
        cred_offer: &CredentialOfferResponse
    ) -> Outcome<()>;
    async fn get_vpd(&self, payload: &OidcUri) -> Outcome<Vpd>;
    fn parse_vpd(&self, vpd_as_string: &str) -> Outcome<Vpd>;
    async fn get_matching_vcs(&self, vpd: &Vpd) -> Outcome<Vec<String>>;
    async fn match_vc4vp(&self, vp_def: Value) -> Outcome<Vec<MatchingVCs>>;
    async fn present_vp(&self, payload: &OidcUri, vcs_id: Vec<String>) -> Outcome<Option<String>>;
}
