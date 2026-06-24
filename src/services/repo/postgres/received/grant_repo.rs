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
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::entities::received::grant;
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::received::RecvGrantRepoTrait;
use crate::types::gnap::grant_request::GrantKind;

pub struct RecvGrantPostgresRepo {
    db: DatabaseConnection,
}

impl RecvGrantPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for RecvGrantPostgresRepo {
    type Entity = grant::Entity;
    type Plan = grant::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl RecvGrantRepoTrait for RecvGrantPostgresRepo {
    async fn get_by_type(&self, kind: GrantKind) -> Outcome<Vec<grant::Model>> {
        grant::Entity::find()
            .filter(grant::Column::Kind.eq(kind))
            .all(self.db())
            .await
            .map_err(|e| Errors::db("Unable to get grants by kind", Some(Box::new(e))))
    }
}
