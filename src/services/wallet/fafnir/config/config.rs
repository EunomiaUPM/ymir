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

use super::FafnirConfigTrait;
use crate::config::traits::{HostsConfigTrait, WalletConfigTrait};
use crate::config::types::{CommonHostsConfig, WalletConfig};
use crate::types::present::{Missing, Present};

pub struct FafnirConfig {
    hosts: CommonHostsConfig,
    wallet: WalletConfig,
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

impl FafnirConfigTrait for FafnirConfig {}

pub struct FafnirConfigBuilder<H, W> {
    hosts: Option<CommonHostsConfig>,
    wallet: Option<WalletConfig>,
    _marker: PhantomData<(H, W)>,
}

impl FafnirConfigBuilder<Missing, Missing> {
    pub fn new() -> Self {
        Self {
            hosts: None,
            wallet: None,
            _marker: PhantomData,
        }
    }
}

impl<H, W> FafnirConfigBuilder<H, W> {
    pub fn hosts(self, hosts: CommonHostsConfig) -> FafnirConfigBuilder<Present, W> {
        FafnirConfigBuilder {
            hosts: Some(hosts),
            wallet: self.wallet,
            _marker: PhantomData,
        }
    }

    pub fn wallet(self, wallet: WalletConfig) -> FafnirConfigBuilder<H, Present> {
        FafnirConfigBuilder {
            hosts: self.hosts,
            wallet: Some(wallet),
            _marker: PhantomData,
        }
    }
}

impl FafnirConfigBuilder<Present, Present> {
    pub fn build(self) -> FafnirConfig {
        FafnirConfig {
            hosts: self.hosts.unwrap(),
            wallet: self.wallet.unwrap(),
        }
    }
}

impl Default for FafnirConfigBuilder<Missing, Missing> {
    fn default() -> Self {
        Self::new()
    }
}
