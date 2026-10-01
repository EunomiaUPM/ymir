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

//! Database-side listings: filters, keyset order and page size resolved by the repositories.

use chrono::{DateTime, Utc};

use crate::types::gnap::GrantStatus;
use crate::types::gnap::grant_request::GrantKind;
use crate::types::participants::ParticipantType;

/// Order a listing runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Where a listing resumes: sort timestamp and id of the last row already returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyset {
    pub timestamp: DateTime<Utc>,
    pub id: Option<String>,
}

/// A page request resolved in the database, ordered by `(timestamp, id)`.
#[derive(Debug, Clone)]
pub struct ListPage<S> {
    pub sort: S,
    pub direction: SortDirection,
    pub after: Option<Keyset>,
    pub limit: u64,
}

/// One page of rows plus how many rows match the filter overall.
#[derive(Debug, Clone)]
pub struct Listed<T> {
    pub items: Vec<T>,
    pub total: u64,
}

/// Timestamp a participant listing is ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantSort {
    SavedAt,
    LastInteraction,
}

/// Participant filter; `ParticipantType::All` matches every type.
#[derive(Debug, Clone)]
pub struct ParticipantListFilter {
    pub participant_type: ParticipantType,
    pub nick_contains: Option<String>,
    pub id_contains: Option<String>,
    pub saved_after: Option<DateTime<Utc>>,
    pub saved_before: Option<DateTime<Utc>>,
}

/// Timestamp a grant listing is ordered by; `Updated` falls back to creation for open grants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantSort {
    Created,
    Updated,
}

/// Filter over grants this connector sent.
#[derive(Debug, Clone)]
pub struct SentGrantListFilter {
    /// Tenant whose grants are listed; `None` lists every tenant (admins).
    pub tenant_id: String,
    pub user_id: String,
    pub kind: GrantKind,
    pub participant_id_contains: Option<String>,
    pub nick_contains: Option<String>,
    pub status: Option<GrantStatus>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// Filter over grants this connector received.
#[derive(Debug, Clone)]
pub struct RecvGrantListFilter {
    /// Tenant whose grants are listed; `None` lists every tenant (admins).
    pub tenant_id: String,
    pub kind: GrantKind,
    pub nick_contains: Option<String>,
    pub status: Option<GrantStatus>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}
