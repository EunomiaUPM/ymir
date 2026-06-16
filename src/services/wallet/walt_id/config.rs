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

use crate::config::traits::{DidConfigTrait, HostsConfigTrait, WalletConfigTrait};
use crate::config::types::DidConfig;
use crate::config::types::{CommonHostsConfig, WalletConfig};

pub struct WaltIdConfig {
    hosts: CommonHostsConfig,
    ssi_wallet_config: WalletConfig,
    did_config: DidConfig,
}

impl WaltIdConfig {
    pub fn new(hosts: CommonHostsConfig, ssi_wallet_config: WalletConfig, did_config: DidConfig) -> Self {
        Self { hosts, ssi_wallet_config, did_config }
    }
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
