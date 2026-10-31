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

//! Paths of the events API.

/// The events themselves, at the root of the API.
pub mod entries {
    pub const ROOT: &str = "/";
    /// Server-sent events.
    pub const STREAM: &str = "/stream";
    /// Read state of the caller.
    pub const UNREAD_COUNT: &str = "/unread-count";
    pub const READ_ALL: &str = "/read-all";
    pub const BY_ID: &str = "/{id}";
    pub const READ: &str = "/{id}/read";
    pub const DELIVERIES: &str = "/{id}/deliveries";
}

pub mod subscriptions {
    pub const PREFIX: &str = "/subscriptions";
    pub const ROOT: &str = "/";
    pub const BY_ID: &str = "/{id}";
}

/// Dead letter queue.
pub mod dlq {
    pub const PREFIX: &str = "/dlq";
    pub const ROOT: &str = "/";
    pub const REPLAY_ALL: &str = "/replay-all";
    pub const BY_ID: &str = "/{id}";
    pub const REPLAY: &str = "/{id}/replay";
}
