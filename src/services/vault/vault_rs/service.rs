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
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{Database, DatabaseConnection};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::info;
use vaultrs::api::sys::requests::EnableEngineRequestBuilder;
use vaultrs::client::{VaultClient, VaultClientSettings, VaultClientSettingsBuilder};
use vaultrs::kv2;
use vaultrs::sys::mount;

use crate::config::traits::DatabaseConfigTrait;
use crate::errors::{Errors, Outcome};
use crate::services::vault::VaultTrait;
use crate::types::secrets::{DbSecrets, PemHelper, StringHelper};
use crate::utils::{expect_from_env, read, read_json};

/// Production Vault service backed by HashiCorp Vault.
///
/// Communicates via HTTP/S using the `vaultrs` crate ecosystem. Manages active engine mounts,
/// checks seals, and pulls raw secrets containing cryptographic material and dynamic DB configurations.
pub struct RealVaultService {
    client: Arc<VaultClient>,
    mount: String,
    vault_path: PathBuf,
    db_path: String,
}

impl RealVaultService {
    pub fn new() -> Outcome<Self> {
        let settings = VaultClientSettingsBuilder::default()
            .build()
            .map_err(|e| Errors::vault("Error creating vault settings", Some(Box::new(e))))?;

        Self::custom(settings)
    }
    pub fn custom(settings: VaultClientSettings) -> Outcome<Self> {
        let mount = std::env::var("VAULT_MOUNT")
            .map_err(|e| Errors::vault("VAULT_MOUNT env var not set", Some(Box::new(e))))?;
        let vault_path = PathBuf::from(
            std::env::var("VAULT_PATH")
                .map_err(|e| Errors::vault("VAULT_PATH env var not set", Some(Box::new(e))))?,
        );
        let db_path = std::env::var("VAULT_APP_DB")
            .map_err(|e| Errors::vault("VAULT_APP_DB env var not set", Some(Box::new(e))))?;
        let client = VaultClient::new(settings)
            .map_err(|e| Errors::vault("Error building custom vault", Some(Box::new(e))))?;

        Ok(Self {
            client: Arc::new(client),
            mount,
            vault_path,
            db_path,
        })
    }
}

#[async_trait]
impl VaultTrait for RealVaultService {
    async fn read<T>(&self, mount: Option<&str>, path: &str) -> Outcome<T>
    where
        T: DeserializeOwned,
    {
        let secret = self.basic_read(mount, path).await?;
        Ok(serde_json::from_value(secret)?)
    }
    async fn basic_read(&self, mount: Option<&str>, path: &str) -> Outcome<Value> {
        let mount = mount.unwrap_or(&self.mount);
        kv2::read(&*self.client, mount, path).await.map_err(|e| {
            Errors::vault(
                format!("Error reading from vault at {mount}/{path}"),
                Some(Box::new(e)),
            )
        })
    }
    async fn write<T>(&self, mount: Option<&str>, path: &str, secret: &T) -> Outcome<()>
    where
        T: Serialize + Send + Sync,
    {
        let mount = mount.unwrap_or(&self.mount);
        kv2::set(&*self.client, mount, path, secret)
            .await
            .map_err(|e| {
                Errors::vault(
                    format!("Error writing to vault at {mount}/{path}"),
                    Some(Box::new(e)),
                )
            })?;

        Ok(())
    }

    async fn write_all_secrets(&self, map: Option<HashMap<String, Value>>) -> Outcome<()> {
        let to_write = match map {
            Some(m) => m,
            None => self.secrets()?,
        };
        self.check_mount().await?;
        for (path, secret) in to_write {
            self.write(None, &path, &secret).await?;
        }
        Ok(())
    }

    async fn check_mount(&self) -> Outcome<()> {
        let existing_mounts = mount::list(&*self.client)
            .await
            .map_err(|e| Errors::vault("Error listing mounts", Some(Box::new(e))))?;

        let mount_path = format!("{}/", self.mount);
        if !existing_mounts.contains_key(&mount_path) {
            let mut opts = HashMap::new();
            opts.insert("version".to_string(), "2".to_string());
            let mut data = EnableEngineRequestBuilder::default();
            let data = data.options(opts);

            mount::enable(&*self.client, &self.mount, "kv", Some(data))
                .await
                .map_err(|e| {
                    Errors::vault(
                        format!("Error creating mount '{}'", self.mount),
                        Some(Box::new(e)),
                    )
                })?;

            info!("Mount '{}' created successfully", self.mount);
        } else {
            info!("Mount '{}' already exists, omitting step", self.mount);
        }
        Ok(())
    }

    async fn get_db_connection<T>(&self, config: &T) -> Outcome<DatabaseConnection>
    where
        T: DatabaseConfigTrait + Send + Sync,
    {
        let db_secrets: DbSecrets = self
            .read(None, &self.db_path)
            .await
            .map_err(|e| Errors::vault("Not able to retrieve env files", Some(Box::new(e))))?;
        Database::connect(config.get_full_db_url(&db_secrets))
            .await
            .map_err(|e| Errors::db("Error connecting to database", Some(Box::new(e))))
    }
}

impl RealVaultService {
    fn insert_json<T>(
        mapa: &mut HashMap<String, Value>,
        to_read: T,
        env: &str,
        required: bool,
    ) -> Outcome<()>
    where
        T: AsRef<Path>,
    {
        let vault_path = expect_from_env(env);
        let db_json = match read_json(to_read) {
            Ok(db_json) => db_json,
            Err(e) => return if required { Err(e) } else { Ok(()) },
        };
        mapa.insert(vault_path, db_json);
        Ok(())
    }
    fn insert_pem<T>(mapa: &mut HashMap<String, Value>, to_read: T, env: &str) -> Outcome<()>
    where
        T: AsRef<Path>,
    {
        let vault_path = expect_from_env(env);
        let data = read(to_read)?;
        let data = serde_json::to_value(&StringHelper::new(data))?;
        mapa.insert(vault_path, data);
        Ok(())
    }

    fn insert_parsed_pem<S, T>(
        mapa: &mut HashMap<String, Value>,
        to_read: T,
        env: &str,
        parser: S,
    ) -> Outcome<()>
    where
        T: AsRef<Path>,
        S: FnOnce(&str) -> Outcome<PemHelper>,
    {
        let vault_path = expect_from_env(env);
        let pem = read(to_read)?;
        let helper = parser(&pem)?;
        let value = serde_json::to_value(&helper)?;
        mapa.insert(vault_path, value);
        Ok(())
    }

    fn secrets(&self) -> Outcome<HashMap<String, Value>> {
        let mut map: HashMap<String, Value> = HashMap::new();
        let config_path = self.vault_path.join("config");
        let secret_path = self.vault_path.join("secrets");

        Self::insert_json(&mut map, secret_path.join("db.json"), "VAULT_APP_DB", true)?;
        Self::insert_json(
            &mut map,
            secret_path.join("wallet.json"),
            "VAULT_APP_WALLET",
            false,
        )?;
        Self::insert_parsed_pem(
            &mut map,
            secret_path.join("private_key.pem"),
            "VAULT_APP_PRIV_KEY",
            PemHelper::priv_from_pem,
        )?;
        Self::insert_parsed_pem(
            &mut map,
            secret_path.join("public_key.pem"),
            "VAULT_APP_PUB_PKEY",
            PemHelper::pub_from_pem,
        )?;
        Self::insert_pem(&mut map, secret_path.join("cert.pem"), "VAULT_APP_CERT")?;

        Self::insert_pem(
            &mut map,
            config_path.join("vault-cert.pem"),
            "VAULT_APP_CLIENT_CERT",
        )?;
        Self::insert_pem(
            &mut map,
            config_path.join("vault-key.pem"),
            "VAULT_APP_CLIENT_KEY",
        )?;
        Self::insert_pem(
            &mut map,
            config_path.join("vault-ca.pem"),
            "VAULT_APP_ROOT_CLIENT_KEY",
        )?;

        Ok(map)
    }
}
