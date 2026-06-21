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
use crate::capabilities::Did;
use crate::errors::{Outcome};
use crate::types::dids::{DidBuilder, DidDocument};
use crate::types::secrets::PemHelper;
use crate::types::wallet::{Identity, WalletInfo};
use crate::types::wallet::{DidModel, KeyModel, VcModel};
use async_trait::async_trait;
use crate::data::entities::shared::participant;
use crate::types::participants::ParticipantType;

#[async_trait]
pub trait WalletTrait: Send + Sync + 'static {
    // BASIC
    async fn link(&self) -> Outcome<participant::Plan>;
    // GET FROM MANAGER (It gives a cloned Value, not a reference)
    async fn get_wallet(&self) -> Outcome<WalletInfo>;
    fn get_did(&self) -> Outcome<Did>;
    fn get_did_doc(&self) -> Outcome<DidDocument>;
    fn get_identity(&self) -> Outcome<Arc<Identity>>;
    // RETRIEVE FROM WALLET
    async fn retrieve_did(&self, id: &str) -> Outcome<DidModel>;
    async fn retrieve_default_did(&self) -> Outcome<DidModel>;
    async fn retrieve_all_dids(&self) -> Outcome<Vec<DidModel>>;
    async fn retrieve_key(&self, id: &str) -> Outcome<KeyModel>;
    async fn retrieve_all_keys(&self) -> Outcome<Vec<KeyModel>>;
    async fn retrieve_vc(&self, id: &str) -> Outcome<VcModel>;
    async fn retrieve_all_vcs(&self) -> Outcome<Vec<VcModel>>;
    // REGISTER STUFF IN WALLET
    async fn register_key(
        &self,
        pem_helper: &PemHelper,
        alias: Option<String>,
    ) -> Outcome<KeyModel>;
    async fn register_did(
        &self,
        did_builder: &DidBuilder,
        keys_id: Vec<String>,
        alias: Option<String>,
    ) -> Outcome<DidModel>;
    async fn set_default_did(&self, did: Did) -> Outcome<()>;
    // DELETE STUFF FROM WALLET
    async fn delete_key(&self, id: &str) -> Outcome<()>;
    async fn delete_did(&self, id: &str) -> Outcome<()>;
    async fn delete_vc(&self, id: &str) -> Outcome<()>;
    // DO STUFF IN WALLET
    async fn process_oid4vci(&self, uri: &str) -> Outcome<()>;
    async fn process_oid4vp(&self, uri: &str) -> Outcome<()>;
    fn get_myself_plan(&self, base_url: String, participant_type: ParticipantType) -> Outcome<participant::Plan> {
        let did = self.get_did()?;
        Ok(participant::Plan {
            participant_id: did.id().to_string(),
            participant_nick: "Myself".to_string(),
            participant_type,
            base_url,
            token: None,
            extra_fields: None,
            is_me: true,
        })
    }
}
