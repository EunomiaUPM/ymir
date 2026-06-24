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

use crate::data::entities::sent::interaction::{Model, Plan};
use crate::services::repo::traits::CrudRepoTrait;
use async_trait::async_trait;

/// Data Repository Contract for Outbound GNAP User Interaction sessions.
///
/// Inherits foundational CRUD layers from [`CrudRepoTrait`]. Tracks and lifecycle-manages 
/// interactive authentication hooks (such as redirect URIs or user codes) demanded by 
/// external Authorization Servers to complete a dynamic grant approval loop.
#[async_trait]
pub trait SentInteractionRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static {}