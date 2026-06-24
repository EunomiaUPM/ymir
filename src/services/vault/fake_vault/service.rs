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

use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::traits::DatabaseConfigTrait;
use crate::errors::{Errors, Outcome};
use crate::services::vault::VaultTrait;
use crate::types::secrets::{DbSecrets, PemHelper, StringHelper};
use crate::utils::{expect_from_env, read, read_json, write_json};

/// Sandbox Vault service backed by the local filesystem.
///
/// Emulates the Key-Value (KV2) behavior of a real Vault instance by translating paths
/// into flat JSON files inside a designated development directory.
pub struct FakeVaultService {
    path: PathBuf,
    db_path: String,
}

impl FakeVaultService {
    pub fn new() -> Outcome<Self> {
        let path = std::env::var("VAULT_PATH")
            .map_err(|e| Errors::vault("VAULT_PATH env var not set", Some(Box::new(e))))?;
        let db_path = std::env::var("VAULT_APP_DB")
            .map_err(|e| Errors::vault("VAULT_APP_DB env var not set", Some(Box::new(e))))?;
        Ok(FakeVaultService {
            path: PathBuf::from(path),
            db_path,
        })
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

    async fn basic_read(&self, _mount: Option<&str>, path: &str) -> Outcome<Value> {
        let path = self.path.join(path);
        read_json(path)
    }

    async fn write<T>(&self, _mount: Option<&str>, path: &str, secret: &T) -> Outcome<()>
    where
        T: Serialize + Send + Sync,
    {
        let path = self.path.join(path);
        write_json(path, secret)
    }

    async fn write_all_secrets(&self, _map: Option<HashMap<String, Value>>) -> Outcome<()> {
        self.write_all_pems()
    }

    async fn check_mount(&self) -> Outcome<()> {
        Ok(())
    }

    async fn get_db_connection<T>(&self, config: &T) -> Outcome<DatabaseConnection>
    where
        T: DatabaseConfigTrait + Send + Sync,
    {
        let path = self.path.join(&self.db_path);

        let db_secrets: DbSecrets = read_json(path)?;
        Database::connect(config.get_full_db_url(&db_secrets))
            .await
            .map_err(|e| Errors::db("Error connecting to database", Some(Box::new(e))))
    }
}

impl FakeVaultService {
    fn write_all_pems(&self) -> Outcome<()> {
        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let pub_key = expect_from_env("VAULT_APP_PUB_PKEY");
        let cert = expect_from_env("VAULT_APP_CERT");

        self.write_parsed_key_pem(&priv_key, PemHelper::priv_from_pem)?;
        self.write_parsed_key_pem(&pub_key, PemHelper::pub_from_pem)?;
        self.write_pem(&cert)
    }
    fn write_pem(&self, json_file: &str) -> Outcome<()> {
        let pem_file = Self::json_to_pem_extension(&json_file);
        let path = self.path.join(pem_file);
        let pem = read(path)?;

        let value = StringHelper::new(pem);

        write_json(self.path.join(json_file), &value)
    }
    fn write_parsed_key_pem<T>(&self, json_file: &str, parser: T) -> Outcome<()>
    where
        T: FnOnce(&str) -> Outcome<PemHelper>,
    {
        let pem_file = Self::json_to_pem_extension(json_file);
        let path = self.path.join(pem_file);
        let pem = read(path)?;
        let value = parser(&pem)?;
        write_json(self.path.join(json_file), &value)
    }
    pub fn json_to_pem_extension(s: &str) -> String {
        s.replace(".json", ".pem")
    }
}
