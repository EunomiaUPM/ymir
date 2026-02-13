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

use super::BasicVerifierConfigTrait;
use crate::config::traits::{HostsConfigTrait, VcConfigTrait};
use crate::config::types::{CommonHostsConfig, VcConfig};
use crate::types::present::{Missing, Present};
use crate::types::vcs::VcType;

pub struct BasicVerifierConfig {
    hosts: CommonHostsConfig,
    is_local: bool,
    api_path: String,
    requested_vcs: Vec<VcType>,
    vc_config: VcConfig
}

impl HostsConfigTrait for BasicVerifierConfig {
    fn hosts(&self) -> &CommonHostsConfig { &self.hosts }
}

impl VcConfigTrait for BasicVerifierConfig {
    fn vc_config(&self) -> &VcConfig { &self.vc_config }
}

impl BasicVerifierConfigTrait for BasicVerifierConfig {
    fn is_local(&self) -> bool { self.is_local }
    fn get_requested_vcs(&self) -> Vec<VcType> { self.requested_vcs.clone() }
    fn get_api_path(&self) -> String { self.api_path.clone() }
}

pub struct BasicVerifierConfigBuilder<H, L, A, V, C> {
    hosts: Option<CommonHostsConfig>,
    is_local: Option<bool>,
    api_path: Option<String>,
    requested_vcs: Option<Vec<VcType>>,
    vc_config: Option<VcConfig>,
    _marker: PhantomData<(H, L, A, V, C)>
}

impl BasicVerifierConfigBuilder<Missing, Missing, Missing, Missing, Missing> {
    pub fn new() -> Self {
        Self {
            hosts: None,
            is_local: None,
            api_path: None,
            requested_vcs: None,
            vc_config: None,
            _marker: PhantomData
        }
    }
}

impl<H, L, A, V, C> BasicVerifierConfigBuilder<H, L, A, V, C> {
    pub fn hosts(
        self,
        hosts: CommonHostsConfig
    ) -> BasicVerifierConfigBuilder<Present, L, A, V, C> {
        BasicVerifierConfigBuilder {
            hosts: Some(hosts),
            is_local: self.is_local,
            api_path: self.api_path,
            requested_vcs: self.requested_vcs,
            vc_config: self.vc_config,
            _marker: PhantomData
        }
    }

    pub fn local(self, is_local: bool) -> BasicVerifierConfigBuilder<H, Present, A, V, C> {
        BasicVerifierConfigBuilder {
            hosts: self.hosts,
            is_local: Some(is_local),
            api_path: self.api_path,
            requested_vcs: self.requested_vcs,
            vc_config: self.vc_config,
            _marker: PhantomData
        }
    }

    pub fn api_path(
        self,
        api_path: impl Into<String>
    ) -> BasicVerifierConfigBuilder<H, L, Present, V, C> {
        BasicVerifierConfigBuilder {
            hosts: self.hosts,
            is_local: self.is_local,
            api_path: Some(api_path.into()),
            requested_vcs: self.requested_vcs,
            vc_config: self.vc_config,
            _marker: PhantomData
        }
    }

    pub fn requested_vcs(
        self,
        vcs: Vec<VcType>
    ) -> BasicVerifierConfigBuilder<H, L, A, Present, C> {
        BasicVerifierConfigBuilder {
            hosts: self.hosts,
            is_local: self.is_local,
            api_path: self.api_path,
            requested_vcs: Some(vcs),
            vc_config: self.vc_config,
            _marker: PhantomData
        }
    }

    pub fn vc_config(self, vc_config: VcConfig) -> BasicVerifierConfigBuilder<H, L, A, V, Present> {
        BasicVerifierConfigBuilder {
            hosts: self.hosts,
            is_local: self.is_local,
            api_path: self.api_path,
            requested_vcs: self.requested_vcs,
            vc_config: Some(vc_config),
            _marker: PhantomData
        }
    }
}

impl BasicVerifierConfigBuilder<Present, Present, Present, Present, Present> {
    pub fn build(self) -> BasicVerifierConfig {
        BasicVerifierConfig {
            hosts: self.hosts.unwrap(),
            is_local: self.is_local.unwrap(),
            api_path: self.api_path.unwrap(),
            requested_vcs: self.requested_vcs.unwrap(),
            vc_config: self.vc_config.unwrap()
        }
    }
}
