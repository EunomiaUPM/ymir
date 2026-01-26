/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use super::super::super::subtraits::{BasicRepoTrait, IssuingTrait};
use crate::data::entities::issuing::{Column, Entity, Model, NewModel};

#[derive(Clone)]
pub struct IssuingRepo {
    db_connection: DatabaseConnection
}

impl IssuingRepo {
    pub fn new(db_connection: DatabaseConnection) -> Self { Self { db_connection } }
}

impl BasicRepoTrait<Entity, NewModel> for IssuingRepo {
    fn db(&self) -> &DatabaseConnection { &self.db_connection }
}

#[async_trait]
impl IssuingTrait for IssuingRepo {
    async fn get_by_pre_auth_code(&self, code: &str) -> anyhow::Result<Model> {
        let to_find = Entity::find().filter(Column::PreAuthCode.eq(code));
        self.help_find(to_find, "code", code).await
    }

    async fn get_by_token(&self, token: &str) -> anyhow::Result<Model> {
        let to_find = Entity::find().filter(Column::Token.eq(token));
        self.help_find(to_find, "token", token).await
    }
}
