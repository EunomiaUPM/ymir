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

use crate::data::entities::shared::issuance;
use crate::data::entities::shared::issuance::Model;
use crate::errors::Outcome;
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::shared::IssuanceRepoTrait;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct IssuancePostgresRepo {
    db: DatabaseConnection,
}

impl IssuancePostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for IssuancePostgresRepo {
    type Entity = issuance::Entity;
    type Plan = issuance::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl IssuanceRepoTrait for IssuancePostgresRepo {
    async fn get_by_pre_auth_code(&self, code: &str) -> Outcome<Model> {
        let query = issuance::Entity::find().filter(issuance::Column::PreAuthCode.eq(code));

        self.basic_filter(query, "pre_auth_code", code).await
    }
    async fn get_by_token(&self, token: &str) -> Outcome<Model> {
        let query = issuance::Entity::find().filter(issuance::Column::Token.eq(token));

        self.basic_filter(query, "token", token).await
    }
}
