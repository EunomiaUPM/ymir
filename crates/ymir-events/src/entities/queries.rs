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

//! Listing filters of the event feed, webhook subscriptions and dead letters.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::paginated_spec::deserialize_opt_bool_from_str_or_bool;
use ymir::query::QueryFilter;

use crate::entities::envelope::{EventCategory, EventOutcome, EventSeverity};
use crate::entities::topic_pattern::TopicPattern;

/// Canonical filter for event feeds, streams and dataspace observability queries.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventFilter {
    /// Topic pattern (`*` matches one segment, `**` any number of segments).
    pub topic: Option<String>,
    /// Crate that published the event, such as `transfer_agent`.
    pub source_crate: Option<String>,
    /// Dataspace identifier.
    pub dataspace_id: Option<String>,
    /// Emitting participant DID/identifier.
    pub participant_id: Option<String>,
    /// Emitting logical service name (e.g. `monolith`, `heimdall`, `dataplane`).
    pub service: Option<String>,
    /// High-level event category.
    pub event_type: Option<EventCategory>,
    /// Severity level filter.
    pub severity: Option<EventSeverity>,
    /// Execution outcome filter.
    pub outcome: Option<EventOutcome>,
    /// Schema type identifier (e.g. `identity.grant.v1`).
    pub payload_type: Option<String>,
    /// Actor identifier filter.
    pub actor_id: Option<String>,
    /// Subject kind filter (`grant`, `transfer_process`, `agreement`, etc.).
    pub subject_kind: Option<String>,
    /// Subject identifier filter.
    pub subject_id: Option<String>,
    /// Cross-flow correlation identifier filter.
    pub correlation_id: Option<String>,
    /// Flow reference filters across dataspace domains.
    pub agreement_id: Option<String>,
    pub transfer_id: Option<String>,
    pub negotiation_id: Option<String>,
    pub grant_id: Option<String>,
    pub dataset_id: Option<String>,
    pub peer_id: Option<String>,
    /// `false` keeps unread events only, `true` read ones (per caller).
    #[serde(default, deserialize_with = "deserialize_opt_bool_from_str_or_bool")]
    pub read: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl EventFilter {
    /// Parsed `topic` pattern; fails on an invalid pattern.
    pub fn topic_pattern(&self) -> Outcome<Option<TopicPattern>> {
        self.topic
            .as_deref()
            .map(|t| TopicPattern::new(t).map_err(|e| Errors::format(BadFormat::Received, e, None)))
            .transpose()
    }
}

impl QueryFilter for EventFilter {
    fn is_empty(&self) -> bool {
        self.topic.is_none()
            && self.source_crate.is_none()
            && self.dataspace_id.is_none()
            && self.participant_id.is_none()
            && self.service.is_none()
            && self.event_type.is_none()
            && self.severity.is_none()
            && self.outcome.is_none()
            && self.payload_type.is_none()
            && self.actor_id.is_none()
            && self.subject_kind.is_none()
            && self.subject_id.is_none()
            && self.correlation_id.is_none()
            && self.agreement_id.is_none()
            && self.transfer_id.is_none()
            && self.negotiation_id.is_none()
            && self.grant_id.is_none()
            && self.dataset_id.is_none()
            && self.peer_id.is_none()
            && self.read.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        self.topic_pattern().map(|_| ())
    }
}

/// Subscription listing filter; `active` accepts `true`/`false` or `1`/`0`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SubscriptionFilter {
    #[serde(default, deserialize_with = "deserialize_opt_bool_from_str_or_bool")]
    pub active: Option<bool>,
}

impl QueryFilter for SubscriptionFilter {
    fn is_empty(&self) -> bool {
        self.active.is_none()
    }
}

/// Dead letter listing filter by status, such as `Unresolved`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DeadLetterFilter {
    pub status: Option<String>,
}

impl QueryFilter for DeadLetterFilter {
    fn is_empty(&self) -> bool {
        self.status.is_none()
    }
}
