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

use crate::data::entities::wallet::key;
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::wallet::KeyRepoTrait;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

pub struct KeyPostgresRepo {
    db: DatabaseConnection,
}

impl KeyPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for KeyPostgresRepo {
    type Entity = key::Entity;
    type Plan = key::Model;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl KeyRepoTrait for KeyPostgresRepo {}
