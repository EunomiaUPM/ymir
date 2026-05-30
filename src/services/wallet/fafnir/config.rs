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

use std::marker::PhantomData;

use crate::config::traits::{DidConfigTrait, HostsConfigTrait, WalletConfigTrait};
use crate::config::types::{CommonHostsConfig, DidConfig, WalletConfig};
use crate::types::present::{Missing, Present};

pub struct FafnirConfig {
    hosts: CommonHostsConfig,
    wallet: WalletConfig,
    did: DidConfig,
}

impl HostsConfigTrait for FafnirConfig {
    fn hosts(&self) -> &CommonHostsConfig {
        &self.hosts
    }
}

impl WalletConfigTrait for FafnirConfig {
    fn wallet_config(&self) -> &WalletConfig {
        &self.wallet
    }
}

impl DidConfigTrait for FafnirConfig {
    fn did_config(&self) -> &DidConfig {
        &self.did
    }
}


pub struct FafnirConfigBuilder<H, W, D> {
    hosts: Option<CommonHostsConfig>,
    wallet: Option<WalletConfig>,
    did: Option<DidConfig>,
    _marker: PhantomData<(H, W, D)>,
}

impl FafnirConfigBuilder<Missing, Missing, Missing> {
    pub fn new() -> Self {
        Self {
            hosts: None,
            wallet: None,
            did: None,
            _marker: PhantomData,
        }
    }
}

impl<H, W, D> FafnirConfigBuilder<H, W, D> {
    pub fn hosts(self, hosts: CommonHostsConfig) -> FafnirConfigBuilder<Present, W, D> {
        FafnirConfigBuilder {
            hosts: Some(hosts),
            wallet: self.wallet,
            did: self.did,
            _marker: PhantomData,
        }
    }

    pub fn wallet(self, wallet: WalletConfig) -> FafnirConfigBuilder<H, Present, D> {
        FafnirConfigBuilder {
            hosts: self.hosts,
            wallet: Some(wallet),
            did: self.did,
            _marker: PhantomData,
        }
    }

    pub fn did(self, did: DidConfig) -> FafnirConfigBuilder<H, W, Present> {
        FafnirConfigBuilder {
            hosts: self.hosts,
            wallet: self.wallet,
            did: Some(did),
            _marker: PhantomData,
        }
    }
}

impl FafnirConfigBuilder<Present, Present, Present> {
    pub fn build(self) -> FafnirConfig {
        FafnirConfig {
            hosts: self.hosts.unwrap(),
            wallet: self.wallet.unwrap(),
            did: self.did.unwrap(),
        }
    }
}

impl Default for FafnirConfigBuilder<Missing, Missing, Missing> {
    fn default() -> Self {
        Self::new()
    }
}
