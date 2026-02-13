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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::IntoActiveSet;
use crate::data::entities::business_mates::{Column, Entity, Model, NewModel};
use crate::errors::{Errors, Outcome};
use crate::services::repo::subtraits::BasicRepoTrait;
use crate::services::repo::subtraits::BusinessMatesRepoTrait;

#[derive(Clone)]
pub struct BusinessMatesRepo {
    db_connection: DatabaseConnection,
}

impl BusinessMatesRepo {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

impl BasicRepoTrait<Entity, NewModel> for BusinessMatesRepo {
    fn db(&self) -> &DatabaseConnection {
        &self.db_connection
    }
}

#[async_trait]
impl BusinessMatesRepoTrait for BusinessMatesRepo {
    async fn get_by_token(&self, token: &str) -> Outcome<Model> {
        let to_find = Entity::find().filter(Column::Token.eq(token));
        self.help_find(to_find, "token", token).await
    }

    async fn force_create(&self, mate: NewModel) -> Outcome<Model> {
        let active_mate = mate.to_active();
        Entity::insert(active_mate)
            .on_conflict(
                OnConflict::column(Column::ParticipantId)
                    .update_columns([Column::Token, Column::LastInteraction])
                    .to_owned(),
            )
            .exec_with_returning(self.db())
            .await
            .map_err(|e| Errors::db("Error forcing creating mate", Some(anyhow::Error::from(e))))
    }
}
