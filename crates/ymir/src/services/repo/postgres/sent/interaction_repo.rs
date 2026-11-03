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
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::data::entities::sent::interaction;
use crate::errors::Outcome;
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::sent::SentInteractionRepoTrait;

pub struct SentInteractionPostgresRepo {
    db: DatabaseConnection,
}

impl SentInteractionPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for SentInteractionPostgresRepo {
    type Entity = interaction::Entity;
    type Plan = interaction::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl SentInteractionRepoTrait for SentInteractionPostgresRepo {
    async fn get_latest_by_grant(&self, grant_id: &str) -> Outcome<interaction::Model> {
        let query = interaction::Entity::find()
            .filter(interaction::Column::GrantId.eq(grant_id))
            .order_by_desc(interaction::Column::CreatedAt);
        self.basic_filter(query, "grant_id", grant_id).await
    }
}
