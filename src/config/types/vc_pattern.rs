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

use serde::{Deserialize, Serialize};

use crate::config::traits::VcConfigTrait;
use crate::types::vcs::{VcModel, W3cDataModelVersion};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VcConfig {
    pub vc_model: VcModel,
    pub w3c_data_model: Option<W3cDataModelVersion>
}

impl VcConfigTrait for VcConfig {
    fn vc_config(&self) -> &VcConfig { self }
}
