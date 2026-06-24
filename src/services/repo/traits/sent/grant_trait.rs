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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::data::entities::sent::grant::{Model, Plan};
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;
use crate::types::gnap::grant_request::GrantKind;
use async_trait::async_trait;

/// Data Repository Contract for Outbound Grant Requests (*Sent Grants*).
///
/// Inherits foundational CRUD layers from [`CrudRepoTrait`]. Tracks the execution state 
/// and polling lifecycles of grant requests sent to external Authorization Servers (AS), 
/// serving as the client-side audit ledger for active security negotiations.
#[async_trait]
pub trait SentGrantRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static {

    /// Filters and gathers sent grants matching a specific intent or operational type.
    ///
    /// Essential for orchestrating background tasks, handling status polling loops 
    /// for pending interactions, or separating credential-issuance grants from standard data access tokens.
    async fn filter_by_type(&self, kind: GrantKind) -> Outcome<Vec<Model>>;
}