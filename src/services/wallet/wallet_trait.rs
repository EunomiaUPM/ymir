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

use crate::data::entities::{mates, minions};
use crate::errors::{Outcome};
use async_trait::async_trait;
use crate::capabilities::Did;
use crate::types::dids::{DidBuilder, DidDocument};
use crate::types::secrets::PemHelper;
use crate::types::wallet::fafnir::{DidEntry, KeyEntry, VcEntry};
use crate::types::wallet::WalletInfo;

#[async_trait]
pub trait WalletTrait: Send + Sync + 'static {
    // BASIC
    async fn link(&self) -> Outcome<(mates::NewModel, minions::NewModel)>;
    // GET FROM MANAGER (It gives a cloned Value, not a reference)
    async fn get_wallet(&self) -> Outcome<WalletInfo>;
    fn get_did(&self) -> Outcome<Did>;
    fn get_did_doc(&self) -> Outcome<DidDocument>;
    // RETRIEVE FROM WALLET
    async fn retrieve_did(&self, id: &str) -> Outcome<DidEntry>;
    async fn retrieve_default_did(&self) -> Outcome<DidEntry>;
    async fn retrieve_all_dids(&self) -> Outcome<Vec<DidEntry>>;
    async fn retrieve_key(&self, id: &str) -> Outcome<KeyEntry>;
    async fn retrieve_all_keys(&self) -> Outcome<Vec<KeyEntry>>;
    async fn retrieve_vc(&self, id: &str) -> Outcome<VcEntry>;
    async fn retrieve_all_vcs(&self) -> Outcome<Vec<VcEntry>>;
    // REGISTER STUFF IN WALLET
    async fn register_key(&self, pem_helper: &PemHelper, alias: Option<String>) -> Outcome<KeyEntry>;
    async fn register_did(&self, did_builder: &DidBuilder, keys_id: Vec<String>, alias: Option<String>) -> Outcome<DidEntry>;
    async fn set_default_did(&self, did: Did) -> Outcome<()>;
    // DELETE STUFF FROM WALLET
    async fn delete_key(&self, id: &str) -> Outcome<()>;
    async fn delete_did(&self, id: &str) -> Outcome<()>;
    async fn delete_vc(&self, id: &str) -> Outcome<()>;
    // DO STUFF IN WALLET
    async fn process_oid4vci(&self, uri: &str) -> Outcome<()>;
    async fn process_oid4vp(&self, uri: &str) -> Outcome<()>;
    fn get_self_mate(&self, base_url: String) -> Outcome<mates::NewModel> {
        let did = self.get_did()?;
        Ok(mates::NewModel {
            participant_id: did.id().to_string(),
            participant_slug: "Myself".to_string(),
            participant_type: "Agent".to_string(),
            base_url,
            token: None,
            extra_fields: None,
            is_me: true,
        })
    }

    fn get_self_minion(&self, base_url: String) -> Outcome<minions::NewModel> {
        let did = self.get_did()?;
        Ok(minions::NewModel {
            participant_id: did.id().to_string(),
            participant_slug: "Myself".to_string(),
            participant_type: "Authority".to_string(),
            base_url: Some(base_url),
            vc_uri: None,
            is_vc_issued: false,
            is_me: true,
        })
    }
}

