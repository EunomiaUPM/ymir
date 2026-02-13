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

use super::WaltIdConfigTrait;
use crate::config::traits::{DidConfigTrait, HostsConfigTrait, WalletConfigTrait};
use crate::config::types::DidConfig;
use crate::config::types::{CommonHostsConfig, WalletConfig};
use crate::types::present::{Missing, Present};

pub struct WaltIdConfig {
    hosts: CommonHostsConfig,
    ssi_wallet_config: WalletConfig,
    did_config: DidConfig,
}

impl HostsConfigTrait for WaltIdConfig {
    fn hosts(&self) -> &CommonHostsConfig {
        &self.hosts
    }
}

impl WalletConfigTrait for WaltIdConfig {
    fn wallet_config(&self) -> &WalletConfig {
        &self.ssi_wallet_config
    }
}

impl DidConfigTrait for WaltIdConfig {
    fn did_config(&self) -> &DidConfig {
        &self.did_config
    }
}

impl WaltIdConfigTrait for WaltIdConfig {}

pub struct WaltIdConfigBuilder<H, W, D> {
    hosts: Option<CommonHostsConfig>,
    ssi_wallet_config: Option<WalletConfig>,
    did_config: Option<DidConfig>,
    _marker: PhantomData<(H, W, D)>,
}

impl WaltIdConfigBuilder<Missing, Missing, Missing> {
    pub fn new() -> Self {
        Self { hosts: None, ssi_wallet_config: None, did_config: None, _marker: PhantomData }
    }
}

impl<H, W, D> WaltIdConfigBuilder<H, W, D> {
    pub fn hosts(self, hosts: CommonHostsConfig) -> WaltIdConfigBuilder<Present, W, D> {
        WaltIdConfigBuilder {
            hosts: Some(hosts),
            ssi_wallet_config: self.ssi_wallet_config,
            did_config: self.did_config,
            _marker: PhantomData,
        }
    }
    pub fn ssi_wallet_config(self, cfg: WalletConfig) -> WaltIdConfigBuilder<H, Present, D> {
        WaltIdConfigBuilder {
            hosts: self.hosts,
            ssi_wallet_config: Some(cfg),
            did_config: self.did_config,
            _marker: PhantomData,
        }
    }

    pub fn did_config(self, cfg: DidConfig) -> WaltIdConfigBuilder<H, W, Present> {
        WaltIdConfigBuilder {
            hosts: self.hosts,
            ssi_wallet_config: self.ssi_wallet_config,
            did_config: Some(cfg),
            _marker: PhantomData,
        }
    }
}

impl WaltIdConfigBuilder<Present, Present, Present> {
    pub fn build(self) -> WaltIdConfig {
        WaltIdConfig {
            hosts: self.hosts.unwrap(),
            ssi_wallet_config: self.ssi_wallet_config.unwrap(),
            did_config: self.did_config.unwrap(),
        }
    }
}
