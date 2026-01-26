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

use anyhow::bail;
use tracing::error;

use crate::config::types::{HostConfig, HostType};
use crate::errors::{ErrorLogTrait, Errors};

pub trait HostsConfigTrait {
    fn http(&self) -> &HostConfig;
    fn grpc(&self) -> Option<&HostConfig>;
    fn graphql(&self) -> Option<&HostConfig>;
    fn get_host(&self, host_type: HostType) -> String {
        let host = match host_type {
            HostType::Http => Self::get_host_helper(Some(self.http()), &host_type.to_string()),
            HostType::Grpc => Self::get_host_helper(self.grpc(), &host_type.to_string()),
            HostType::Graphql => Self::get_host_helper(self.graphql(), &host_type.to_string()),
        };
        host.expect("Failed to get host")
    }

    fn get_host_without_protocol(&self, host_type: HostType) -> String {
        let host = match host_type {
            HostType::Http => self.http(),
            HostType::Grpc => self.grpc().expect("Failed to get grpc host"),
            HostType::Graphql => self.graphql().expect("Failed to get graphql host"),
        };
        host.get_host_without_protocol()
    }

    fn get_weird_port(&self, host_type: HostType) -> String {
        let host = match host_type {
            HostType::Http => self.http(),
            HostType::Grpc => self.grpc().expect("Failed to get grpc host"),
            HostType::Graphql => self.graphql().expect("Failed to get graphql host"),
        };
        host.get_weird_port()
    }

    fn get_host_helper(host: Option<&HostConfig>, module: &str) -> anyhow::Result<String> {
        match host {
            Some(host) => Ok(host.get_host()),
            None => {
                let error = Errors::module_new(module);
                error!("{}", error.log());
                bail!(error)
            }
        }
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
}
