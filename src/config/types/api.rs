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
use crate::config::traits::ApiConfigTrait;

/// Technical exposure matrix defining versioning constraints and specification locations.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ApiConfig {
    /// Canonical version identifier segment used for route dispatch structuring (e.g., "v1").
    pub version: String,
    /// Absolute or relative file-system track path pointing to the local OpenAPI specification sheet asset.
    pub openapi_path: String,
}

impl ApiConfigTrait for ApiConfig {
    fn api(&self) -> &ApiConfig {
        self
    }
}