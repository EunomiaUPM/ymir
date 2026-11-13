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

//! Event store repository port and UI read-state projection wrapper.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};
use ymir::oauth::OwnerScope;
use ymir::paginated_spec::{Page, Sort};

use crate::entities::envelope::EventEnvelope;
use crate::entities::queries::EventFilter;

/// Stored immutable event combined with the caller's personal `read_at` projection.
#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub envelope: EventEnvelope,
    pub read_at: Option<DateTime<Utc>>,
}

impl StoredEvent {
    /// Wraps an unread envelope.
    pub fn unread(envelope: EventEnvelope) -> Self {
        Self {
            envelope,
            read_at: None,
        }
    }
}

/// Failures of the event store, mapped onto `Errors`.
#[derive(Debug, Error)]
pub enum EventRepoError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Event not found: {0}")]
    NotFound(String),
}

impl RepoIntoErrors for EventRepoError {}

/// Append-only store of canonical v2 events with separated reader state.
#[mockall::automock]
#[async_trait]
pub trait EventStoreRepo: Send + Sync + 'static {
    async fn insert_event(&self, event: &EventEnvelope) -> Outcome<()>;

    async fn get_event_by_id(
        &self,
        scope: &OwnerScope,
        id: &Urn,
        reader_id: Option<String>,
    ) -> Outcome<Option<StoredEvent>>;

    /// A page of events plus the total count of the filtered set.
    async fn list_events(
        &self,
        scope: &OwnerScope,
        filter: &EventFilter,
        reader_id: Option<String>,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<StoredEvent>, u64)>;

    async fn count_unread(&self, scope: &OwnerScope, reader_id: Option<String>) -> Outcome<u64>;

    /// Marks one event read, or every unread one in scope when `id` is `None`; returns how many.
    async fn mark_read(
        &self,
        scope: &OwnerScope,
        reader_id: Option<String>,
        id: Option<Urn>,
    ) -> Outcome<u64>;
}
