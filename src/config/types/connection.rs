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

use serde::{Deserialize, Serialize};
use crate::config::traits::ConnectionConfigTrait;

/// Deployment deployment boundaries and infrastructure environment configuration tracks.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConnectionConfig {
    /// Flag indicating if enforcement settings should lock parameters under production conditions.
    pub is_prod: bool,
    /// Flag indicating whether secure key storage interacts with standard real Vault hardware infrastructure.
    pub is_vault_real: bool,
    /// Flag checking if communication nodes are routed via reverse proxy TLS terminators.
    pub has_tls_proxy: bool,
}

impl ConnectionConfigTrait for ConnectionConfig {
    fn connection(&self) -> &ConnectionConfig {
        self
    }
}