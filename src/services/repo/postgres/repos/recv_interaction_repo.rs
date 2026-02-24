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
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::entities::recv_interaction::{Column, Entity, Model, NewModel};
use crate::errors::{Errors, Outcome};
use crate::services::repo::subtraits::BasicRepoTrait;
use crate::services::repo::subtraits::RecvInteractionTrait;

pub struct RecvInteractionRepo {
    db_connection: DatabaseConnection
}

impl RecvInteractionRepo {
    pub fn new(db_connection: DatabaseConnection) -> Self { Self { db_connection } }
}

impl BasicRepoTrait<Entity, NewModel> for RecvInteractionRepo {
    fn db(&self) -> &DatabaseConnection { &self.db_connection }
}

#[async_trait]
impl RecvInteractionTrait for RecvInteractionRepo {
    async fn get_by_reference(&self, reference: &str) -> Outcome<Model> {
        let to_find = Entity::find().filter(Column::InteractRef.eq(reference));
        self.help_find(to_find, "reference", reference).await
    }

    async fn get_by_cont_id(&self, cont_id: &str) -> Outcome<Model> {
        let to_find = Entity::find().filter(Column::ContinueId.eq(cont_id));
        self.help_find(to_find, "cont_id", cont_id).await
    }

    async fn get_by_some_id(&self, some_id: &str) -> Outcome<Option<Model>> {
        Entity::find_by_id(some_id)
            .one(self.db())
            .await
            .map_err(|e| Errors::db("Error finding some model", Some(anyhow::Error::from(e))))
    }
}
