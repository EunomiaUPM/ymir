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

use serde::de::DeserializeOwned;

use crate::errors::Outcome;
use crate::utils::{parse_yaml, read};

/// Loading an agent config from a YAML file.
pub trait ConfigLoader: Sized + DeserializeOwned {
    /// Reads the agent's config; by default the whole file is this config.
    fn load(env_file: &str) -> Outcome<Self> {
        Self::local_load(env_file)
    }

    /// Reads the file as this config alone; a relative path is resolved from the working directory.
    fn local_load(env_file: &str) -> Outcome<Self> {
        parse_yaml(&read(env_file)?)
    }
}
