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
use crate::data::entities::shared::issuance;
use crate::services::repo::traits::CrudRepoTrait;
use crate::data::entities::shared::issuance::{Model, Plan};
use crate::errors::Outcome;

#[async_trait]
pub trait IssuanceRepoTrait: CrudRepoTrait<Model, Plan> + Send + Sync + 'static
{
    async fn get_by_pre_auth_code(&self, code: &str) -> Outcome<Model>;
    async fn get_by_token(&self, token: &str) -> Outcome<Model>;
}