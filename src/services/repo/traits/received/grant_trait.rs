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
use crate::types::listing::{GrantSort, ListPage, Listed, RecvGrantListFilter};
use async_trait::async_trait;

/// Data Repository Contract for Inbound GNAP Grant Requests (*Received Grants*).
///
/// Inherits foundational CRUD layers from [`CrudRepoTrait`]. Acts as the core ledger
/// for an Authorization Server (AS), tracking incoming authorization requests pending negotiation.
#[async_trait]
pub trait RecvGrantRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static {
    /// Lists one page of grants matching `filter`, filtered and paged in the database.
    async fn find_page(
        &self,
        filter: &RecvGrantListFilter,
        page: &ListPage<GrantSort>,
    ) -> Outcome<Listed<Model>>;
}

#[cfg(feature = "mock")]
mockall::mock! {
    pub RecvGrantRepoTrait {}

    #[async_trait]
    impl CrudRepoTrait<Model, Plan> for RecvGrantRepoTrait {
        async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> Outcome<Vec<Model>>;
        async fn get_by_id(&self, id: &str) -> Outcome<Model>;
        async fn create(&self, plan: Plan) -> Outcome<Model>;
        async fn update(&self, model: Model) -> Outcome<Model>;
        async fn delete(&self, id: &str) -> Outcome<()>;
    }

    #[async_trait]
    impl RecvGrantRepoTrait for RecvGrantRepoTrait {
        async fn find_page(&self, filter: &RecvGrantListFilter, page: &ListPage<GrantSort>) -> Outcome<Listed<Model>>;
    }
}
