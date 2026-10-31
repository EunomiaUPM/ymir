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

//! Config logging, YAML parsing and shallow JSON merge.

use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::info;

use crate::errors::{Errors, Outcome};

/// Logs the config as a table.
pub fn show_table(config: &impl Serialize) -> Outcome<()> {
    let table = json_to_table::json_to_table(
        &serde_json::to_value(config)
            .map_err(|e| Errors::parse("Error with config table", Some(Box::new(e))))?,
    )
    .collapse()
    .to_string();
    info!("Current Config:\n{}", table);
    Ok(())
}

/// Parses YAML text (not a path, despite the name).
pub fn parse_yaml<T: DeserializeOwned>(path: &str) -> Outcome<T> {
    serde_norway::from_str(path)
        .map_err(|e| Errors::parse("Unable to parse config file", Some(Box::new(e))))
}

/// Overwrites the top-level keys of `base` with those of `patch`; non-objects are ignored.
pub fn json_merge(base: &mut serde_json::Value, patch: serde_json::Value) {
    if let (serde_json::Value::Object(base_map), serde_json::Value::Object(patch_map)) =
        (base, patch)
    {
        for (k, v) in patch_map {
            base_map.insert(k, v);
        }
    }
}
