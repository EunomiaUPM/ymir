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
use crate::data::entities::wallet::did;
use crate::errors::Outcome;
use crate::services::repo::postgres::{BasicPostgresRepo};
use crate::services::repo::traits::CrudRepoTrait;
use crate::services::repo::traits::wallet::DidRepoTrait;

pub struct DidPostgresRepo {
    db: DatabaseConnection,
}

impl DidPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for DidPostgresRepo {
    type Entity = did::Entity;
    type Plan = did::Model;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl DidRepoTrait for DidPostgresRepo {
    async fn get_by_did(&self, did: &str) -> Outcome<did::Model> {
        let query = did::Entity::find()
            .filter(did::Column::Did.eq(did));
        self.basic_filter(query, "did", did).await
    }

    async fn get_default(&self) -> Outcome<did::Model> {
        let query = did::Entity::find()
            .filter(did::Column::Default.eq(true));
        self.basic_filter(query, "default", "true").await
    }

    async fn set_default(&self, id: &str) -> Outcome<did::Model> {
        let mut def_model = self.get_default().await?;
        def_model.default = false;
        self.basic_update(def_model).await?;
        let mut def_model = self.basic_get_by_id(id).await?;
        def_model.default = true;
        self.update(def_model).await
    }
}