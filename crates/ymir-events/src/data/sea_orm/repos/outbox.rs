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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! SeaORM implementation of [`EventOutboxRepo`].

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use ymir::errors::{Errors, Outcome};

use crate::data::repo::outbox::EventOutboxRepo;
use crate::data::sea_orm::orm::outbox;
use crate::entities::outbox::{OutboxRecord, OutboxStatus};

#[derive(Clone)]
pub struct SeaOrmOutboxRepo {
    db: DatabaseConnection,
}

impl SeaOrmOutboxRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EventOutboxRepo for SeaOrmOutboxRepo {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn enqueue(&self, record: &OutboxRecord) -> Outcome<()> {
        let active = outbox::ActiveModel::from_domain(record);
        outbox::Entity::insert(active)
            .exec_without_returning(&self.db)
            .await
            .map_err(|e| Errors::db("failed to enqueue event outbox record", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_due_pending(&self, now: DateTime<Utc>, limit: u64) -> Outcome<Vec<OutboxRecord>> {
        let models = outbox::Entity::find()
            .filter(outbox::Column::Status.eq(OutboxStatus::Pending.as_str()))
            .filter(outbox::Column::NextRetryAt.lte(now.naive_utc()))
            .order_by_asc(outbox::Column::NextRetryAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to query due outbox records", Some(Box::new(e))))?;
        Ok(models.into_iter().map(outbox::Model::into_domain).collect())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn mark_delivered(&self, id: &str, attempts: u32) -> Outcome<()> {
        let now = Utc::now().naive_utc();
        outbox::Entity::update_many()
            .col_expr(
                outbox::Column::Status,
                Expr::value(OutboxStatus::Delivered.as_str()),
            )
            .col_expr(outbox::Column::Attempts, Expr::value(attempts as i32))
            .col_expr(outbox::Column::DeliveredAt, Expr::value(now))
            .filter(outbox::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| Errors::db("failed to mark outbox record delivered", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn reschedule(
        &self,
        id: &str,
        attempts: u32,
        next_retry_at: DateTime<Utc>,
        error: &str,
    ) -> Outcome<()> {
        outbox::Entity::update_many()
            .col_expr(outbox::Column::Attempts, Expr::value(attempts as i32))
            .col_expr(
                outbox::Column::NextRetryAt,
                Expr::value(next_retry_at.naive_utc()),
            )
            .col_expr(outbox::Column::LastError, Expr::value(error.to_string()))
            .filter(outbox::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| Errors::db("failed to reschedule outbox record", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn mark_dead_letter(&self, id: &str, attempts: u32, error: &str) -> Outcome<()> {
        outbox::Entity::update_many()
            .col_expr(
                outbox::Column::Status,
                Expr::value(OutboxStatus::DeadLetter.as_str()),
            )
            .col_expr(outbox::Column::Attempts, Expr::value(attempts as i32))
            .col_expr(outbox::Column::LastError, Expr::value(error.to_string()))
            .filter(outbox::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                Errors::db(
                    "failed to move outbox record to dead letter",
                    Some(Box::new(e)),
                )
            })?;
        Ok(())
    }
}
