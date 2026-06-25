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

use crate::data::entities::received::grant::{Model, Plan};
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;
use crate::types::gnap::grant_request::GrantKind;
use async_trait::async_trait;

/// Data Repository Contract for Inbound GNAP Grant Requests (*Received Grants*).
///
/// Inherits foundational CRUD layers from [`CrudRepoTrait`]. Acts as the core ledger
/// for an Authorization Server (AS), tracking incoming authorization requests pending negotiation.
#[async_trait]
pub trait RecvGrantRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static {
    /// Filters incoming grants by their specific operational request nature ([`GrantKind`]).
    async fn get_by_type(&self, kind: GrantKind) -> Outcome<Vec<Model>>;
}
