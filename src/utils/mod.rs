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

use crate::errors::{ErrorLogTrait, Errors};
use crate::types::errors::BadFormat;
use anyhow::bail;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::Path;
use std::{env, fs};
use tracing::error;

pub fn read<P>(path: P) -> anyhow::Result<String>
where
    P: AsRef<Path>,
{
    let path_ref = path.as_ref();
    match fs::read_to_string(path_ref) {
        Ok(data) => Ok(data),
        Err(e) => {
            let error = Errors::read_new(&path_ref.display().to_string(), &e.to_string());
            error!("{}", error.log());
            bail!(error)
        }
    }
}

pub fn read_json<T, P>(path: P) -> anyhow::Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let data = read(path)?;
    let json = serde_json::from_str(&data)?;
    Ok(json)
}

pub fn expect_from_env(env: &str) -> String {
    let result = env::var(env);
    let data = match result {
        Ok(data) => Some(data),
        Err(e) => {
            let error = Errors::env_new(format!("{} not found -> {}", &env, e.to_string()));
            error!("{}", error.log());
            None
        }
    };
    data.expect("Error with env variable")
}

pub fn get_claim(claims: &Value, path: Vec<&str>) -> anyhow::Result<String> {
    let mut node = claims;
    let field = path.last().unwrap_or(&"unknown");
    for key in path.iter() {
        node = match node.get(key) {
            Some(data) => data,
            None => {
                let error =
                    Errors::format_new(BadFormat::Received, &format!("Missing field '{}'", key));
                error!("{}", error.log());
                bail!(error)
            }
        };
    }
    validate_data(node, field)
}

pub fn get_opt_claim(claims: &Value, path: Vec<&str>) -> anyhow::Result<Option<String>> {
    let mut node = claims;
    let field = path.last().unwrap_or(&"unknown");
    for key in path.iter() {
        node = match node.get(key) {
            Some(data) => data,
            None => return Ok(None),
        };
    }
    let data = validate_data(node, field)?;
    Ok(Some(data))
}

fn validate_data(node: &Value, field: &str) -> anyhow::Result<String> {
    match node.as_str() {
        Some(data) => Ok(data.to_string()),
        None => {
            let error =
                Errors::format_new(BadFormat::Received, &format!("Field '{}' not a string", field));
            error!("{}", error.log());
            bail!(error)
        }
    }
}
