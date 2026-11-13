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

//! Outbox repository port for durable downstream broker relay (`OutboxRelay`).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::outbox::OutboxRecord;

/// Failures of the event outbox repository.
#[derive(Debug, Error)]
pub enum OutboxRepoError {
    #[error("Outbox database error: {0}")]
    Database(String),
}

impl RepoIntoErrors for OutboxRepoError {}

/// Durable spool for events awaiting delivery to external message backbones such as Kafka.
#[mockall::automock]
#[async_trait]
pub trait EventOutboxRepo: Send + Sync + 'static {
    async fn enqueue(&self, record: &OutboxRecord) -> Outcome<()>;

    async fn get_due_pending(&self, now: DateTime<Utc>, limit: u64) -> Outcome<Vec<OutboxRecord>>;

    async fn mark_delivered(&self, id: &str, attempts: u32) -> Outcome<()>;

    async fn reschedule(
        &self,
        id: &str,
        attempts: u32,
        next_retry_at: DateTime<Utc>,
        error: &str,
    ) -> Outcome<()>;

    async fn mark_dead_letter(&self, id: &str, attempts: u32, error: &str) -> Outcome<()>;
}
