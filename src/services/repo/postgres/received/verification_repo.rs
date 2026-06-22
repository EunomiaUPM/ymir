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
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};

use crate::data::entities::received::{verification};
use crate::errors::{Outcome};
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::received::RecvVerificationRepoTrait;

pub struct RecvVerificationPostgresRepo {
    db: DatabaseConnection,
}

impl RecvVerificationPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for RecvVerificationPostgresRepo {
    type Entity = verification::Entity;
    type Plan = verification::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl RecvVerificationRepoTrait for RecvVerificationPostgresRepo {
    async fn get_by_state(&self, state: &str) -> Outcome<verification::Model> {
        let query = verification::Entity::find()
            .filter(verification::Column::State.eq(state));

        self.basic_filter(query, "state", state).await
    }
}
