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
use std::{format, vec};
use async_trait::async_trait;
use sea_orm::Condition;
use sea_orm::prelude::Expr;
use crate::data::entities::wallet::vc;
use crate::services::repo::traits::CrudRepoTrait;
use crate::data::entities::wallet::vc::{Model};
use crate::errors::Outcome;
use crate::types::vcs::{InputDescriptor, VcType};

#[async_trait]
pub trait VcRepoTrait: CrudRepoTrait<Model, Model> + Send + Sync + 'static
{
    async fn filter_by_type(&self, r#type: VcType) -> Outcome<Vec<Model>>;
    async fn filter_by_desc(&self, input_descriptor: &InputDescriptor ) -> Outcome<Vec<Model>>;
}