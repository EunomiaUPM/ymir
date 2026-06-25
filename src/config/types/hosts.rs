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

use crate::config::traits::{HostsConfigTrait, SingleHostTrait};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Data representation wrapping single transport perimeter networking specs.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HostConfig {
    /// Target routing scheme protocol (e.g. "http", "https").
    pub protocol: String,
    /// Absolute domain address string, local node identifier, or IP matrix.
    pub url: String,
    /// Network ingress point exposed to multi-tenant ecosystems.
    pub port: Option<String>,
    /// Private cluster deployment container transport mapping boundary.
    pub internal_port: Option<String>,
}

impl SingleHostTrait for HostConfig {
    fn host(&self) -> &HostConfig {
        self
    }
}

/// Unified host transport layout controlling node exposure surfaces.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommonHostsConfig {
    /// Primary entry endpoint managing standard Web interactions.
    pub http: HostConfig,
    /// Complementary endpoint handling low-latency high-throughput internal RPC tasks.
    pub grpc: Option<HostConfig>,
    /// Complementary data query subsystem interface configuration mappings.
    pub graphql: Option<HostConfig>,
}

impl HostsConfigTrait for CommonHostsConfig {
    fn hosts(&self) -> &CommonHostsConfig {
        self
    }
}

/// Supported transport pipeline taxonomies within data space nodes.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum HostType {
    Http,
    Grpc,
    Graphql,
}

impl Display for HostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            HostType::Http => "http",
            HostType::Grpc => "grpc",
            HostType::Graphql => "graphql",
        };
        write!(f, "{}", str)
    }
}
