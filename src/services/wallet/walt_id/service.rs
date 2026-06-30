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
use tokio::sync::RwLock;

use async_trait::async_trait;

use super::super::WalletTrait;
use crate::capabilities::Did;
use crate::data::entities::wallet::{did, key, vc};
use crate::errors::Outcome;
use crate::types::dids::DidDocument;
use crate::types::wallet::{DidSearch, Identity, WalletInfo};

pub struct WaltIdService {}

impl WaltIdService {
    pub async fn new() -> Outcome<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl WalletTrait for WaltIdService {
    async fn link(&self) -> Outcome<()> {
        todo!()
    }

    async fn get_wallet(&self) -> Outcome<WalletInfo> {
        todo!()
    }

    async fn get_did(&self) -> Outcome<Did> {
        todo!()
    }

    async fn get_did_doc(&self) -> Outcome<DidDocument> {
        todo!()
    }

    fn get_identity(&self) -> Arc<RwLock<Identity>> {
        todo!()
    }

    async fn retrieve_did(&self, _search: DidSearch) -> Outcome<did::Model> {
        todo!()
    }

    async fn retrieve_default_did(&self) -> Outcome<did::Model> {
        todo!()
    }

    async fn retrieve_all_dids(&self) -> Outcome<Vec<did::Model>> {
        todo!()
    }

    async fn retrieve_key(&self, _id: &str) -> Outcome<key::Model> {
        todo!()
    }

    async fn retrieve_all_keys(&self) -> Outcome<Vec<key::Model>> {
        todo!()
    }

    async fn retrieve_vc(&self, _id: &str) -> Outcome<vc::Model> {
        todo!()
    }

    async fn retrieve_all_vcs(&self) -> Outcome<Vec<vc::Model>> {
        todo!()
    }

    async fn register_key(&self, _plan: key::Plan) -> Outcome<key::Model> {
        todo!()
    }

    async fn register_did(&self, _plan: did::Plan) -> Outcome<did::Model> {
        todo!()
    }

    async fn store_vc(&self, _plan: vc::Plan) -> Outcome<vc::Model> {
        todo!()
    }

    async fn set_default_did(&self, _search: DidSearch) -> Outcome<did::Model> {
        todo!()
    }

    async fn add_key_to_did(
        &self,
        _search: DidSearch,
        _key_id: String,
    ) -> Outcome<did::Model> {
        todo!()
    }

    async fn remove_key_from_did(
        &self,
        _search: DidSearch,
        _key_id: String,
    ) -> Outcome<did::Model> {
        todo!()
    }

    async fn set_default_key(
        &self,
        _search: DidSearch,
        _key_id: String,
    ) -> Outcome<did::Model> {
        todo!()
    }

    async fn delete_key(&self, _id: &str) -> Outcome<()> {
        todo!()
    }

    async fn delete_did(&self, _search: DidSearch) -> Outcome<()> {
        todo!()
    }

    async fn delete_vc(&self, _id: &str) -> Outcome<()> {
        todo!()
    }

    async fn process_oid4vci(&self, _uri: &str) -> Outcome<()> {
        todo!()
    }

    async fn process_oid4vp(&self, _uri: &str) -> Outcome<()> {
        todo!()
    }
}
