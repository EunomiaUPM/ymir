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

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

use crate::config::traits::DatabaseConfigTrait;
use crate::errors::Errors;

/// Core database configuration tracking endpoints and driver backends.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    /// Engine architecture driver flag.
    pub db_type: DbType,
    /// Targeted database location domain or network IP.
    pub url: String,
    /// Ingress connection port vector.
    pub port: String,
}

impl DatabaseConfigTrait for DatabaseConfig {
    fn db(&self) -> &DatabaseConfig {
        self
    }
}

// ===== DATABASE ENGINE TAXONOMY ==================================================================

/// Supported data engine drivers inside identity nodes.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DbType {
    Postgres,
    Mysql,
    Sqlite,
    Mongo,
    Memory,
}

impl Display for DbType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbType::Postgres => write!(f, "postgres"),
            DbType::Mysql => write!(f, "mysql"),
            DbType::Sqlite => write!(f, "sqlite"),
            DbType::Mongo => write!(f, "mongodb"),
            DbType::Memory => write!(f, "memory"),
        }
    }
}

impl FromStr for DbType {
    type Err = Errors;

    /// Parses environment tokens into structural database engines. Fixes previous copy-paste bug.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgres" => Ok(DbType::Postgres),
            "mysql" => Ok(DbType::Mysql),
            "sqlite" => Ok(DbType::Sqlite),
            "mongodb" => Ok(DbType::Mongo),
            "memory" => Ok(DbType::Memory),
            _ => Err(Errors::parse("unknown database type", None)),
        }
    }
}
