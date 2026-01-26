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

use std::marker::PhantomData;
use tracing::error;

use super::WaltIdConfigTrait;
use crate::config::traits::SingleHostTrait;
use crate::errors::{ErrorLogTrait, Errors};
use crate::types::dids::did_config::DidConfig;
use crate::types::dids::did_type::DidType;
use crate::types::present::{Missing, Present};
use crate::types::wallet::WalletConfig;

pub struct WaltIdConfig {
    ssi_wallet_config: WalletConfig,
    did_config: DidConfig,
}

impl WaltIdConfigTrait for WaltIdConfig {
    fn get_raw_wallet_config(&self) -> WalletConfig {
        self.ssi_wallet_config.clone()
    }
    fn get_wallet_api_url(&self) -> String {
        self.ssi_wallet_config.api.get_host()
    }
    fn get_did_type(&self) -> DidType {
        self.did_config.r#type.clone()
    }
    fn get_did_web_path(&self) -> Option<String> {
        match self.did_config.r#type {
            DidType::Web => self.did_config.did_web_options.as_ref()?.path.clone(),
            _ => {
                let error = Errors::module_new("didweb");
                error!("{}", error.log());
                None
            }
        }
    }
    fn get_did_web_domain(&self) -> String {
        let domain = match self.did_config.r#type {
            DidType::Web => {
                Some(self.did_config.did_web_options.as_ref().expect("didweb").domain.clone())
            }
            _ => {
                let error = Errors::module_new("didweb");
                error!("{}", error.log());
                None
            }
        };

        domain.expect("didweb")
    }
}

pub struct WaltIdConfigBuilder<W, D> {
    ssi_wallet_config: Option<WalletConfig>,
    did_config: Option<DidConfig>,
    _marker: PhantomData<(W, D)>,
}

impl WaltIdConfigBuilder<Missing, Missing> {
    pub fn new() -> Self {
        Self { ssi_wallet_config: None, did_config: None, _marker: PhantomData }
    }
}

impl<W, D> WaltIdConfigBuilder<W, D> {
    pub fn ssi_wallet_config(self, cfg: WalletConfig) -> WaltIdConfigBuilder<Present, D> {
        WaltIdConfigBuilder {
            ssi_wallet_config: Some(cfg),
            did_config: self.did_config,
            _marker: PhantomData,
        }
    }

    pub fn did_config(self, cfg: DidConfig) -> WaltIdConfigBuilder<W, Present> {
        WaltIdConfigBuilder {
            ssi_wallet_config: self.ssi_wallet_config,
            did_config: Some(cfg),
            _marker: PhantomData,
        }
    }
}

impl WaltIdConfigBuilder<Present, Present> {
    pub fn build(self) -> WaltIdConfig {
        WaltIdConfig {
            ssi_wallet_config: self.ssi_wallet_config.unwrap(),
            did_config: self.did_config.unwrap(),
        }
    }
}
