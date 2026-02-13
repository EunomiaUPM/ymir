/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::config::types::{CommonHostsConfig, HostConfig, HostType};
use crate::errors::Errors;

pub trait HostsConfigTrait {
    fn hosts(&self) -> &CommonHostsConfig;
    fn http(&self) -> &HostConfig {
        &self.hosts().http
    }
    fn grpc(&self) -> Option<&HostConfig> {
        self.hosts().grpc.as_ref()
    }
    fn graphql(&self) -> Option<&HostConfig> {
        self.hosts().graphql.as_ref()
    }
    fn get_host(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_host()
    }

    fn get_host_without_protocol(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_host_without_protocol()
    }

    fn get_weird_port(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_weird_port()
    }
    fn get_tls_port(&self, host_type: HostType) -> String {
        self.get_helper(host_type).get_tls_port()
    }

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

pub trait SingleHostTrait {
    fn host(&self) -> &HostConfig;
    fn get_host(&self) -> String {
        match self.host().port.as_ref() {
            Some(port) => format!("{}://{}:{}", self.host().protocol, self.host().url, port),
            None => format!("{}://{}", self.host().protocol, self.host().url),
        }
    }
    fn get_host_without_protocol(&self) -> String {
        match self.host().port.as_ref() {
            Some(port) => {
                format!("{}:{}", self.host().url, port)
            }
            None => self.host().url.clone(),
        }
    }
    fn get_weird_port(&self) -> String {
        match self.host().port.as_ref() {
            Some(port) => format!(":{}", port),
            None => "".to_string(),
        }
    }
    fn get_tls_port(&self) -> String {
        self.host().port.clone().unwrap_or_else(|| "443".to_string())
    }
}
