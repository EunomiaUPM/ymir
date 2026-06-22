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

use crate::config::traits::{HostsConfigTrait};
use crate::config::types::{CommonHostsConfig};
use crate::types::vcs::VcType;

pub struct VerifierConfig {
    hosts: CommonHostsConfig,
    api_path: String,
    requested_vcs: Vec<VcType>,
}

impl VerifierConfig {
    pub fn new(
        hosts: CommonHostsConfig,
        api_path: String,
        requested_vcs: Vec<VcType>,
    ) -> Self {
        Self {
            hosts,
            api_path,
            requested_vcs,
        }
    }

    pub fn get_requested_vcs(&self) -> &[VcType] {
        &self.requested_vcs
    }
    pub fn get_api_path(&self) -> &str {
        &self.api_path
    }
}

impl HostsConfigTrait for VerifierConfig {
    fn hosts(&self) -> &CommonHostsConfig {
        &self.hosts
    }
}


impl VerifierConfig {}
