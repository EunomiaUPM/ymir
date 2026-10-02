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

use crate::data::entities::wallet::vc::Model;
use crate::errors::Outcome;
use crate::services::repo::traits::CrudRepoTrait;
use crate::types::vcs::{InputDescriptor, VcType};

use async_trait::async_trait;

/// Data Repository Contract for Verifiable Credentials (VCs) lifecycle management.
///
/// Inherits foundational CRUD operations from [`CrudRepoTrait`] and supplies
/// cryptographic matching filters to resolve verification requests within the SSI ecosystem.
#[async_trait]
pub trait VcRepoTrait: CrudRepoTrait<Model, Model> + Send + Sync + 'static {
    // ===== EXTENDED SEMANTIC QUERIES =============================================================

    /// Retrieves all stored Verifiable Credentials matching a specific schemas or contextual type.
    ///
    /// Useful for grouping credentials prior to rendering UI elements or compiling stats.
    async fn filter_by_type(&self, r#type: VcType) -> Outcome<Vec<Model>>;

    /// Evaluates stored credentials against constraints declared inside a DIF Presentation Exchange structure.
    ///
    /// This method is crucial during OpenID4VP challenge resolutions, enabling the Wallet to scan
    /// fields (such as JSONPaths, issuers, or claim patterns) to determine which VCs are eligible
    /// to satisfy the verifier's requested evaluation criteria.
    async fn filter_by_desc(&self, input_descriptor: &InputDescriptor) -> Outcome<Vec<Model>>;
}

#[cfg(feature = "mock")]
mockall::mock! {
    pub VcRepoTrait {}

    #[async_trait]
    impl CrudRepoTrait<Model, Model> for VcRepoTrait {
        async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> Outcome<Vec<Model>>;
        async fn get_by_id(&self, id: &str) -> Outcome<Model>;
        async fn create(&self, plan: Model) -> Outcome<Model>;
        async fn update(&self, model: Model) -> Outcome<Model>;
        async fn delete(&self, id: &str) -> Outcome<()>;
    }

    #[async_trait]
    impl VcRepoTrait for VcRepoTrait {
        async fn filter_by_type(&self, r#type: VcType) -> Outcome<Vec<Model>>;
        async fn filter_by_desc(&self, input_descriptor: &InputDescriptor) -> Outcome<Vec<Model>>;
    }
}
