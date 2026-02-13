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
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::IntoActiveSet;
use crate::data::entities::recv_verification::{Column, Entity, Model, NewModel};
use crate::errors::{Errors, Outcome};
use crate::services::repo::subtraits::BasicRepoTrait;
use crate::services::repo::subtraits::RecvVerificationTrait;

#[derive(Clone)]
pub struct RecvVerificationRepo {
    db_connection: DatabaseConnection,
}

impl RecvVerificationRepo {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

impl BasicRepoTrait<Entity, NewModel> for RecvVerificationRepo {
    fn db(&self) -> &DatabaseConnection {
        &self.db_connection
    }
}

#[async_trait]
impl RecvVerificationTrait for RecvVerificationRepo {
    async fn get_by_state(&self, state: &str) -> Outcome<Model> {
        let to_find = Entity::find().filter(Column::State.eq(state));
        self.help_find(to_find, "state", state).await
    }

    async fn create_from_basic(&self, model: Model) -> Outcome<Model> {
        let active_model = model.to_active();
        active_model
            .insert(self.db())
            .await
            .map_err(|e| Errors::db("Error creating from basic", Some(anyhow::Error::from(e))))
    }
}
