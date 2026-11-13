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

//! Persistent outbox records for reliable downstream broker delivery (Kafka `OutboxRelay`).

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Lifecycle state of a spooled external sink delivery attempt.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutboxStatus {
    #[default]
    Pending,
    Delivered,
    DeadLetter,
}

impl OutboxStatus {
    /// Canonical PascalCase status label stored in `event_outbox`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Delivered => "Delivered",
            Self::DeadLetter => "DeadLetter",
        }
    }
}

impl Display for OutboxStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for OutboxStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "delivered" => Ok(Self::Delivered),
            "deadletter" | "dead_letter" | "dead-letter" => Ok(Self::DeadLetter),
            other => Err(format!("unknown outbox status '{other}'")),
        }
    }
}

/// Persisted failed/pending event waiting for asynchronous broker replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxRecord {
    pub id: String,
    pub sink_name: String,
    pub event_id: String,
    pub destination: String,
    pub partition_key: String,
    pub envelope_json: serde_json::Value,
    pub attempts: u32,
    pub next_retry_at: DateTime<Utc>,
    pub last_error: Option<String>,
    pub status: OutboxStatus,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}
