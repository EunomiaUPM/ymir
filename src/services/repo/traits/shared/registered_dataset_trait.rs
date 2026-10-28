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

use crate::data::entities::shared::registered_dataset::Model;
use crate::errors::Outcome;
use async_trait::async_trait;

/// The datasets the catalog has registered with the AS; it always sends them all at once.
#[cfg_attr(feature = "mock", mockall::automock)]
#[async_trait]
pub trait RegisteredDatasetRepoTrait: Send + Sync + 'static {
    /// Leaves exactly `datasets` registered.
    async fn replace_all(&self, datasets: Vec<Model>) -> Outcome<()>;
    /// The registered datasets among `ids`.
    async fn get_by_ids(&self, ids: &[String]) -> Outcome<Vec<Model>>;
    /// Every registered dataset; locations are matched by path, which the caller does.
    async fn get_all(&self) -> Outcome<Vec<Model>>;
}
