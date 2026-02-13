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

use crate::config::types::{DidConfig, DidWebOptions};
use crate::types::dids::did_type::DidType;

pub trait DidConfigTrait {
    fn did_config(&self) -> &DidConfig;
    fn get_did(&self) -> &str {
        &self.did_config().did
    }
    fn get_did_type(&self) -> &DidType {
        &self.did_config().r#type
    }
    fn get_did_web_options(&self) -> Option<&DidWebOptions> {
        self.did_config().did_web_options.as_ref()
    }
}
