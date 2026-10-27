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
use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::data::entities::received::verification;
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::received::RecvVerificationRepoTrait;
use crate::types::verification::VerificationStatus;

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
        let query = verification::Entity::find().filter(verification::Column::State.eq(state));

        self.basic_filter(query, "state", state).await
    }

    async fn get_latest_by_grant(&self, grant_id: &str) -> Outcome<verification::Model> {
        let query = verification::Entity::find()
            .filter(verification::Column::GrantId.eq(grant_id))
            .order_by_desc(verification::Column::CreatedAt);
        self.basic_filter(query, "grant_id", grant_id).await
    }

    async fn get_all_by_grant(&self, grant_id: &str) -> Outcome<Vec<verification::Model>> {
        verification::Entity::find()
            .filter(verification::Column::GrantId.eq(grant_id))
            .order_by_asc(verification::Column::CreatedAt)
            .all(self.db())
            .await
            .map_err(|e| Errors::db("Unable to get the verifications of a grant", Some(Box::new(e))))
    }

    async fn expire_stale_pending(
        &self,
        created_before: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Outcome<u64> {
        let changes = verification::ActiveModel {
            status: ActiveValue::Set(VerificationStatus::Expired),
            ended_at: ActiveValue::Set(Some(now)),
            ..Default::default()
        };
        let result = verification::Entity::update_many()
            .set(changes)
            .filter(verification::Column::Status.eq(VerificationStatus::Pending))
            .filter(verification::Column::CreatedAt.lt(created_before))
            .exec(self.db())
            .await
            .map_err(|e| Errors::db("Unable to expire stale verifications", Some(Box::new(e))))?;
        Ok(result.rows_affected)
    }
}
