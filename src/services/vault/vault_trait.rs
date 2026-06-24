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

/// Secure Secret Vault abstraction.
///
/// Provides an interface for reading and writing sensitive data, validating
/// cryptographic storage mounts, and provisioning dynamic database connections.
#[async_trait]
pub trait VaultTrait: Send + Sync + 'static {
    // ===== SECRET MANAGEMENT (READ / WRITE) ======================================================

    /// Retrieves and deserializes a secret from the specified vault path.
    ///
    /// If no `mount` is provided, a default system secret engine mount will be assumed.
    async fn read<T>(&self, mount: Option<&str>, path: &str) -> Outcome<T>
    where
        T: DeserializeOwned + Send;

    /// Retrieves a raw, unstructured JSON [`Value`] secret from the vault.
    async fn basic_read(&self, mount: Option<&str>, path: &str) -> Outcome<Value>;

    /// Stores a serializable secret into the specified vault path.
    async fn write<T>(&self, mount: Option<&str>, path: &str, secret: &T) -> Outcome<()>
    where
        T: Serialize + Send + Sync;

    // ===== PROVISIONING & CONFIGURATION ==========================================================

    /// Seeds multiple secrets into the vault at once.
    ///
    /// Useful during environment initialization or bootstrap sequences.
    async fn write_all_secrets(&self, map: Option<HashMap<String, Value>>) -> Outcome<()>;

    /// Validates that the backend storage mount is initialized, unsealed, and reachable.
    async fn check_mount(&self) -> Outcome<()>;

    // ===== INFRASTRUCTURE CONNECTIONS ============================================================

    /// Establishes a database connection using configuration parameters retrieved via the vault.
    ///
    /// This may leverage dynamic credentials engines to safely acquire short-lived database
    /// access tokens before wrapping the resulting connection pool into a [`DatabaseConnection`].
    async fn get_db_connection<T>(&self, config: &T) -> Outcome<DatabaseConnection>
    where
        T: DatabaseConfigTrait + Send + Sync;
}
