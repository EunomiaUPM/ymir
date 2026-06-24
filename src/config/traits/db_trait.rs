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

use crate::config::types::{DatabaseConfig, DbType};
use crate::types::secrets::DbSecrets;

/// Shared behavior for component configurations provisioning structural database connections.
pub trait DatabaseConfigTrait {
    // ===== EXTRACTION ANCHORS ====================================================================

    /// Returns a backing reference to the root database configuration model.
    fn db(&self) -> &DatabaseConfig;

    // ===== URL STRING ENGINE =====================================================================

    /// Assembles the complete canonical connection string injected into data mapping layers (e.g., Sea-ORM).
    ///
    /// Automatically isolates volatile properties like passwords using runtime decoupled [`DbSecrets`].
    fn get_full_db_url(&self, db_secrets: &DbSecrets) -> String {
        let db_config = self.db();
        match db_config.db_type {
            DbType::Memory => ":memory:".to_string(),
            _ => format!(
                "{}://{}:{}@{}:{}/{}",
                db_config.db_type,
                db_secrets.user,
                db_secrets.password,
                db_config.url,
                db_config.port,
                db_secrets.name
            ),
        }
    }
}