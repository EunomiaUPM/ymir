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

use crate::config::traits::{DidConfigTrait, HostsConfigTrait};
use crate::config::types::{CommonHostsConfig, DidConfig};
use crate::services::issuer::basic::config::config_trait::BasicIssuerConfigTrait;
use crate::types::present::{Missing, Present};

pub struct BasicIssuerConfig {
    hosts: CommonHostsConfig,
    is_local: bool,
    api_path: String,
    did_config: DidConfig
}

impl HostsConfigTrait for BasicIssuerConfig {
    fn hosts(&self) -> &CommonHostsConfig { &self.hosts }
}

impl DidConfigTrait for BasicIssuerConfig {
    fn did_config(&self) -> &DidConfig { &self.did_config }
}

impl BasicIssuerConfigTrait for BasicIssuerConfig {
    fn is_local(&self) -> bool { self.is_local }

    fn get_api_path(&self) -> String { self.api_path.clone() }
}

pub struct BasicIssuerConfigBuilder<H, L, A, D> {
    hosts: Option<CommonHostsConfig>,
    is_local: Option<bool>,
    api_path: Option<String>,
    did_config: Option<DidConfig>,
    _marker: PhantomData<(H, L, A, D)>
}

impl BasicIssuerConfigBuilder<Missing, Missing, Missing, Missing> {
    pub fn new() -> Self {
        Self { hosts: None, is_local: None, api_path: None, did_config: None, _marker: PhantomData }
    }
}

impl<H, L, A, D> BasicIssuerConfigBuilder<H, L, A, D> {
    pub fn hosts(self, hosts: CommonHostsConfig) -> BasicIssuerConfigBuilder<Present, L, A, D> {
        BasicIssuerConfigBuilder {
            hosts: Some(hosts),
            is_local: self.is_local,
            api_path: self.api_path,
            did_config: self.did_config,
            _marker: PhantomData
        }
    }

    pub fn local(self, is_local: bool) -> BasicIssuerConfigBuilder<H, Present, A, D> {
        BasicIssuerConfigBuilder {
            hosts: self.hosts,
            is_local: Some(is_local),
            api_path: self.api_path,
            did_config: self.did_config,
            _marker: PhantomData
        }
    }

    pub fn api_path(
        self,
        api_path: impl Into<String>
    ) -> BasicIssuerConfigBuilder<H, L, Present, D> {
        BasicIssuerConfigBuilder {
            hosts: self.hosts,
            is_local: self.is_local,
            api_path: Some(api_path.into()),
            did_config: self.did_config,
            _marker: PhantomData
        }
    }

    pub fn did_config(self, did_config: DidConfig) -> BasicIssuerConfigBuilder<H, L, A, Present> {
        BasicIssuerConfigBuilder {
            hosts: self.hosts,
            is_local: self.is_local,
            api_path: self.api_path,
            did_config: Some(did_config),
            _marker: PhantomData
        }
    }
}

impl BasicIssuerConfigBuilder<Present, Present, Present, Present> {
    pub fn build(self) -> BasicIssuerConfig {
        BasicIssuerConfig {
            hosts: self.hosts.unwrap(),
            is_local: self.is_local.unwrap(),
            api_path: self.api_path.unwrap(),
            did_config: self.did_config.unwrap()
        }
    }
}
