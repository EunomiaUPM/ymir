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

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::super::fake_vault::FakeVaultService;
use super::super::vault_rs::RealVaultService;
use crate::config::traits::DatabaseConfigTrait;
use crate::errors::Outcome;
use crate::services::vault::VaultTrait;

pub enum VaultService {
    Real(RealVaultService),
    Fake(FakeVaultService)
}

#[async_trait]
impl VaultTrait for VaultService {
    async fn read<T>(&self, mount: Option<&str>, path: &str) -> Outcome<T>
    where
        T: DeserializeOwned + Send
    {
        match self {
            VaultService::Real(v) => v.read(mount, path).await,
            VaultService::Fake(v) => v.read(mount, path).await
        }
    }

    async fn basic_read(&self, mount: &str, path: &str) -> Outcome<Value> {
        match self {
            VaultService::Real(v) => v.basic_read(mount, path).await,
            VaultService::Fake(v) => v.basic_read(mount, path).await
        }
    }

    async fn write<T>(&self, mount: Option<&str>, path: &str, secret: &T) -> Outcome<()>
    where
        T: Serialize + Send + Sync
    {
        match self {
            VaultService::Real(v) => v.write(mount, path, secret).await,
            VaultService::Fake(v) => v.write(mount, path, secret).await
        }
    }

    async fn write_all_secrets(&self, map: Option<HashMap<String, Value>>) -> Outcome<()> {
        match self {
            VaultService::Real(v) => v.write_all_secrets(map).await,
            VaultService::Fake(v) => v.write_all_secrets(map).await
        }
    }

    async fn write_local_secrets(&self, map: Option<HashMap<String, Value>>) -> Outcome<()> {
        match self {
            VaultService::Real(v) => v.write_local_secrets(map).await,
            VaultService::Fake(v) => v.write_local_secrets(map).await
        }
    }

    async fn check_mount(&self) -> Outcome<()> {
        match self {
            VaultService::Real(v) => v.check_mount().await,
            VaultService::Fake(v) => v.check_mount().await
        }
    }

    async fn get_db_connection<T>(&self, config: &T) -> DatabaseConnection
    where
        T: DatabaseConfigTrait + Send + Sync
    {
        match self {
            VaultService::Real(v) => v.get_db_connection(config).await,
            VaultService::Fake(v) => v.get_db_connection(config).await
        }
    }
}
