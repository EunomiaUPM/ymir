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
use crate::services::repo::traits::CrudRepoTrait;
use crate::data::entities::wallet::did::Model;
use crate::errors::Outcome;

#[async_trait]
pub trait DidRepoTrait: CrudRepoTrait<Model, Model> + Send + Sync + 'static
{
    async fn get_by_did(&self, did: &str) -> Outcome<Model>;
    async fn get_default(&self) -> Outcome<Model>;
    async fn set_default(&self, id: &str) -> Outcome<Model>;
}