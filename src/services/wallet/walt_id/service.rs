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

use std::sync::{Arc};
use tokio::sync::{RwLock};

use async_trait::async_trait;

use super::super::WalletTrait;
use crate::capabilities::Did;
use crate::data::entities::wallet::did::Model;
use crate::errors::Outcome;
use crate::types::dids::{DidBuilder, DidDocument};
use crate::types::secrets::PemHelper;
use crate::types::wallet::{Identity, WalletInfo};

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

    async fn retrieve_did(&self, _id: &str) -> Outcome<Model> {
        todo!()
    }

    async fn retrieve_default_did(&self) -> Outcome<Model> {
        todo!()
    }

    async fn retrieve_all_dids(&self) -> Outcome<Vec<Model>> {
        todo!()
    }

    async fn retrieve_key(&self, _id: &str) -> Outcome<crate::data::entities::wallet::key::Model> {
        todo!()
    }

    async fn retrieve_all_keys(&self) -> Outcome<Vec<crate::data::entities::wallet::key::Model>> {
        todo!()
    }

    async fn retrieve_vc(&self, _id: &str) -> Outcome<crate::data::entities::wallet::vc::Model> {
        todo!()
    }

    async fn retrieve_all_vcs(&self) -> Outcome<Vec<crate::data::entities::wallet::vc::Model>> {
        todo!()
    }

    async fn register_key(
        &self,
        _pem_helper: &PemHelper,
        _alias: Option<String>,
    ) -> Outcome<crate::data::entities::wallet::key::Model> {
        todo!()
    }

    async fn register_did(
        &self,
        _did_builder: &DidBuilder,
        _keys_id: Vec<String>,
        _alias: Option<String>,
    ) -> Outcome<Model> {
        todo!()
    }

    async fn store_vc(&self, _vc: String) -> Outcome<crate::data::entities::wallet::vc::Model> {
        todo!()
    }

    async fn set_default_did(&self, _did: Did) -> Outcome<Model> {
        todo!()
    }

    async fn delete_key(&self, _id: &str) -> Outcome<()> {
        todo!()
    }

    async fn delete_did(&self, _id: &str) -> Outcome<()> {
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
