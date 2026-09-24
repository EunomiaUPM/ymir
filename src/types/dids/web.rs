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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WebDid {
    id: String,
    #[serde(flatten)]
    config: WebDidConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WebDidConfig {
    pub domain: String,
    pub path: Option<String>,
    pub port: Option<String>,
}

impl WebDid {
    pub fn new(
        id: impl Into<String>,
        domain: impl Into<String>,
        path: Option<String>,
        port: Option<String>,
    ) -> WebDid {
        WebDid {
            id: id.into(),
            config: WebDidConfig {
                domain: domain.into(),
                path,
                port,
            },
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn domain(&self) -> &str {
        &self.config.domain
    }
    pub fn path(&self) -> &Option<String> {
        &self.config.path
    }
    pub fn port(&self) -> &Option<String> {
        &self.config.port
    }

    pub fn get_web_url(&self) -> String {
        let port = match self.port().as_ref() {
            Some(port) => format!(":{port}"),
            None => "".to_string(),
        };
        let protocol = match self.domain() {
            "127.0.0.1" | "localhost" | "host.docker.internal" => "http",
            _ => "https",
        };
        if let Some(path) = &self.path() {
            format!("{}://{}{}/{}/did.json", protocol, self.domain(), port, path)
        } else {
            format!(
                "{}://{}{}/.well-known/did.json",
                protocol,
                self.domain(),
                port
            )
        }
    }
}
