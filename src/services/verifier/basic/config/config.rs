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
use super::BasicVerifierConfigTrait;
use crate::config::traits::SingleHostTrait;
use crate::config::types::HostConfig;
use crate::types::vcs::VcType;

pub struct BasicVerifierConfig {
    host: HostConfig,
    is_local: bool,
    api_path: String,
    requested_vcs: Vec<VcType>,
}

impl BasicVerifierConfigTrait for BasicVerifierConfig {
    fn get_host(&self) -> String {
        self.host.get_host()
    }
    fn get_host_without_protocol(&self) -> String {
        self.host.get_host_without_protocol()
    }
    fn is_local(&self) -> bool {
        self.is_local
    }
    fn get_requested_vcs(&self) -> Vec<VcType> {
        self.requested_vcs.clone()
    }
    fn get_api_path(&self) -> String {
        self.api_path.clone()
    }
}
