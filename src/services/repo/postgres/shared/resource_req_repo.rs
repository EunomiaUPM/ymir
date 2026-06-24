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
use sea_orm::DatabaseConnection;

use crate::data::entities::shared::resource_req;
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::shared::ResourceReqRepoTrait;

pub struct ResourceReqPostgresRepo {
    db: DatabaseConnection,
}

impl ResourceReqPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for ResourceReqPostgresRepo {
    type Entity = resource_req::Entity;
    type Plan = resource_req::Model;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl ResourceReqRepoTrait for ResourceReqPostgresRepo {}
