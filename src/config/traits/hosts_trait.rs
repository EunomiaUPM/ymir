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

use crate::config::types::{CommonHostsConfig, HostConfig, HostType};
use crate::errors::Errors;

/// Shared behavior for component matrices managing multi-transport endpoints.
pub trait HostsConfigTrait {
    // ===== EXTRACTION ANCHORS ====================================================================

    /// Returns a backing reference to the root centralized host matrix.
    fn hosts(&self) -> &CommonHostsConfig;

    /// Isolates the default required HTTP web layer network configurations.
    fn http(&self) -> &HostConfig {
        &self.hosts().http
    }

    /// Isolates the optional gRPC microservice transport configuration boundaries.
    fn grpc(&self) -> Option<&HostConfig> {
        self.hosts().grpc.as_ref()
    }

    /// Isolates the optional GraphQL data query interface configuration parameters.
    fn graphql(&self) -> Option<&HostConfig> {
        self.hosts().graphql.as_ref()
    }

    // ===== STRING RESOLUTION WORKFLOWS ===========================================================

    /// Computes the complete exterior URI format containing target protocol schemes.
    fn get_host(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_host()
    }

    /// Computes the internal boundary URI matching specialized network parameters.
    fn get_internal_host(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_internal_host()
    }

    /// Resolves the raw domain name layout stripped from standard transport scheme prefixes.
    fn get_host_without_protocol(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_host_without_protocol()
    }

    /// Recovers the external canonical secure gateway TLS port bindings.
    fn get_tls_port(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_tls_port()
    }

    /// Recovers the internal secure private microservice network layer ports.
    fn get_internal_port(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_internal_port()
    }

    // ===== ROUTING LOOKUP HELPERS ================================================================

    /// Internally maps individual transport taxonomies against current host configuration tracks.
    ///
    /// # Panics
    /// Dispatches an unrecoverable stack frame panic via [`Errors::ModuleNotActiveError`]
    /// if the chosen target transport configuration route remains undefined on host systems.
    fn get_helper(&self, host_type: HostType) -> &HostConfig {
        let host = match host_type {
            HostType::Http => Some(self.http()),
            HostType::Grpc => self.grpc(),
            HostType::Graphql => self.graphql(),
        };

        host.ok_or_else(|| Errors::not_active(&format!("{} host is not defined", host_type), None))
            .expect(&format!("{} host is not defined", host_type))
    }
}

/// Structural behavior defining extraction rules over a single isolated host envelope.
pub trait SingleHostTrait {
    // ===== EXTRACTION ANCHORS ====================================================================

    /// References the baseline structural host schema targets.
    fn host(&self) -> &HostConfig;

    // ===== METRIC COMPUTATION ENGINE =============================================================

    /// Resolves the complete outward facing web service address layout.
    fn get_host(&self) -> String {
        match self.host().port.as_ref() {
            Some(port) => format!("{}://{}:{}", self.host().protocol, self.host().url, port),
            None => format!("{}://{}", self.host().protocol, self.host().url),
        }
    }

    /// Resolves the backchannel mesh cluster network service address structure.
    fn get_internal_host(&self) -> String {
        format!(
            "{}://{}:{}",
            self.host().protocol,
            self.host().url,
            self.host().get_internal_port()
        )
    }

    /// Strips standard protocol headers, isolating network address names.
    fn get_host_without_protocol(&self) -> String {
        match self.host().port.as_ref() {
            Some(port) => format!("{}:{}", self.host().url, port),
            None => self.host().url.clone(),
        }
    }

    /// Returns designated external ports falling back to the standard `443` secure TLS baseline.
    fn get_tls_port(&self) -> String {
        self.host()
            .port
            .clone()
            .unwrap_or_else(|| "443".to_string())
    }

    /// Returns inner container ports, defaulting to standard TLS port boundaries when unmapped.
    fn get_internal_port(&self) -> String {
        self.host()
            .internal_port
            .clone()
            .unwrap_or_else(|| self.get_tls_port())
    }
}
