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
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};

use crate::data::entities::received::interaction;
use crate::data::entities::received::interaction::Model;
use crate::errors::Outcome;
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::received::RecvInteractionRepoTrait;

pub struct RecvInteractionRepo {
    db: DatabaseConnection,
}

impl RecvInteractionRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for RecvInteractionRepo {
    type Entity = interaction::Entity;
    type Plan = interaction::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl RecvInteractionRepoTrait for RecvInteractionRepo {
    async fn get_by_cont_id(&self, cont_id: &str) -> Outcome<Model> {
        let query = interaction::Entity::find()
            .filter(interaction::Column::ContinueId.eq(cont_id));

        self.basic_filter(query, "cont_id", cont_id).await
    }
}
