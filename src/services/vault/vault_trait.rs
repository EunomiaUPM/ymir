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
    async fn basic_read(&self, mount: Option<&str>, path: &str) -> Outcome<Value>;
    async fn write<T>(&self, mount: Option<&str>, path: &str, secret: &T) -> Outcome<()>
    where
        T: Serialize + Send + Sync;
    /// Push secrets to the vault.
    ///
    /// Real impl: copies the full local secret set (including vault TLS material)
    /// to the remote vault, honoring `map` if provided. Use during bootstrap of a
    /// production deployment.
    ///
    /// Fake impl: ignores `map` and re-packages the local `.pem` files into JSON
    /// blobs at their configured paths inside `VAULT_PATH`, because the Fake
    /// vault is the local filesystem itself.
    async fn write_all_secrets(&self, map: Option<HashMap<String, Value>>) -> Outcome<()>;

    /// Push only the application secrets (no vault TLS material).
    ///
    /// Real impl: writes the application secret subset (db.json, wallet.json,
    /// private/public keys, cert) to the remote vault, honoring `map` if
    /// provided.
    ///
    /// Fake impl: same behavior as `write_all_secrets` — `map` is ignored and
    /// the local `.pem` files are re-packaged as JSON.
    async fn write_local_secrets(&self, map: Option<HashMap<String, Value>>) -> Outcome<()>;
    async fn check_mount(&self) -> Outcome<()>;

    async fn get_db_connection<T>(&self, config: &T) -> Outcome<DatabaseConnection>
    where
        T: DatabaseConfigTrait + Send + Sync;
}
