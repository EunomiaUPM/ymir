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

pub mod fake_vault;
pub mod global;
pub mod vault_rs;
pub mod vault_trait;

pub use global::VaultService;
pub use vault_trait::VaultTrait;

use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::errors::{Errors, Outcome};

/// Shared SQLx/SeaORM pool configuration used by vault database connectors.
pub(crate) struct DbPoolConnector;

impl DbPoolConnector {
    /// Connects to Postgres with a pre-warmed pool and hot-path SQL logging disabled.
    pub(crate) async fn connect(url: String) -> Outcome<DatabaseConnection> {
        let mut options = ConnectOptions::new(url);
        options
            .min_connections(2)
            .max_connections(25)
            .connect_timeout(Duration::from_secs(5))
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(1800))
            .sqlx_logging(false);
        Database::connect(options)
            .await
            .map_err(|e| Errors::db("Error connecting to database", Some(Box::new(e))))
    }
}
