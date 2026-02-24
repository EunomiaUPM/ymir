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
use std::path::PathBuf;

use async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::traits::DatabaseConfigTrait;
use crate::errors::Outcome;
use crate::services::vault::VaultTrait;
use crate::types::secrets::DbSecrets;
use crate::utils::{expect_from_env, read_json};

pub struct FakeVaultService {
    path: PathBuf,
}

impl FakeVaultService {
    pub fn new() -> FakeVaultService {
        let path = PathBuf::from(expect_from_env("VAULT_PATH"));

        FakeVaultService { path }
    }
}

#[async_trait]
impl VaultTrait for FakeVaultService {
    async fn read<T>(&self, _mount: Option<&str>, path: &str) -> Outcome<T>
    where
        T: DeserializeOwned + Send,
    {
        let path = self.path.join(path);
        read_json(path)
    }

    async fn basic_read(&self, _mount: &str, path: &str) -> Outcome<Value> {
        let path = self.path.join(path);
        read_json(path)
    }

    async fn write<T>(&self, _mount: Option<&str>, _path: &str, _secret: &T) -> Outcome<()>
    where
        T: Serialize + Send + Sync,
    {
        Ok(())
    }

    async fn write_all_secrets(&self, _map: Option<HashMap<String, Value>>) -> Outcome<()> {
        Ok(())
    }

    async fn write_local_secrets(&self, _map: Option<HashMap<String, Value>>) -> Outcome<()> {
        Ok(())
    }

    async fn check_mount(&self) -> Outcome<()> {
        Ok(())
    }

    async fn get_db_connection<T>(&self, config: &T) -> DatabaseConnection
    where
        T: DatabaseConfigTrait + Send + Sync,
    {
        let db_path = expect_from_env("VAULT_APP_DB");
        let path = self.path.join(db_path);

        let db_secrets: DbSecrets = read_json(path).expect("VAULT_app secret can't be read");
        Database::connect(config.get_full_db_url(&db_secrets))
            .await
            .expect("Database can't connect")
    }
}
