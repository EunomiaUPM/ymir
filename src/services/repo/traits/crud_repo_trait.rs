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

use async_trait::async_trait;
use crate::errors::Outcome;

#[async_trait]
pub trait CrudRepoTrait<M, P>: Send + Sync + 'static
where
    M: Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> Outcome<Vec<M>>;
    async fn get_by_id(&self, id: &str) -> Outcome<M>;
    async fn create(&self, plan: P) -> Outcome<M>;
    async fn update(&self, model: M) -> Outcome<M>;
    async fn delete(&self, id: &str) -> Outcome<()>;
}

