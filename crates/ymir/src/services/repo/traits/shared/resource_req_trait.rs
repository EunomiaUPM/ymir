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

use crate::data::entities::shared::resource_req::Model;
#[cfg(feature = "mock")]
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;
use async_trait::async_trait;

/// Data Repository Contract for Protected Resource Requests.
///
/// Inherits foundational CRUD capabilities from [`CrudRepoTrait`]. Tracks, historical logs,
/// and updates the lifecycle state of rich authorization requests initiated by clients
/// seeking access to transactional components or credential management subsystems.
#[async_trait]
pub trait ResourceReqRepoTrait: CrudRepoTrait<Model, Model> + Send + Sync + 'static {}

#[cfg(feature = "mock")]
mockall::mock! {
    pub ResourceReqRepoTrait {}

    #[async_trait]
    impl CrudRepoTrait<Model, Model> for ResourceReqRepoTrait {
        async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> Outcome<Vec<Model>>;
        async fn get_by_id(&self, id: &str) -> Outcome<Model>;
        async fn create(&self, plan: Model) -> Outcome<Model>;
        async fn update(&self, model: Model) -> Outcome<Model>;
        async fn delete(&self, id: &str) -> Outcome<()>;
    }

    #[async_trait]
    impl ResourceReqRepoTrait for ResourceReqRepoTrait {}
}
