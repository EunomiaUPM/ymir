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

use std::collections::HashMap;
use std::path::Path;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::traits::DatabaseConfigTrait;
use crate::errors::Outcome;

#[async_trait]
pub trait VaultTrait: Send + Sync + 'static {
    async fn read<T>(&self, mount: Option<&str>, path: &str) -> Outcome<T>
    where
        T: DeserializeOwned + Send;
    async fn basic_read(&self, mount: &str, path: &str) -> Outcome<Value>;
    async fn write<T>(&self, mount: Option<&str>, path: &str, secret: &T) -> Outcome<()>
    where
        T: Serialize + Send + Sync;
    async fn write_all_secrets(&self, map: Option<HashMap<String, Value>>) -> Outcome<()>;
    fn secrets() -> Outcome<HashMap<String, Value>>;
    async fn write_local_secrets(&self, map: Option<HashMap<String, Value>>) -> Outcome<()>;
    fn local_secrets() -> Outcome<HashMap<String, Value>>;
    async fn check_mount(&self) -> Outcome<()>;
    fn insert_json<T>(
        mapa: &mut HashMap<String, Value>,
        to_read: T,
        env: &str,
        required: bool
    ) -> Outcome<()>
    where
        T: AsRef<Path>;
    fn insert_pem<T>(mapa: &mut HashMap<String, Value>, to_read: T, env: &str) -> Outcome<()>
    where
        T: AsRef<Path>;

    async fn get_db_connection<T>(&self, config: &T) -> DatabaseConnection
    where
        T: DatabaseConfigTrait + Send + Sync;
}
