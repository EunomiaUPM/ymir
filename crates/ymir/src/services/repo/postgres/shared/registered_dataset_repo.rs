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
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait,
};

use crate::data::entities::shared::registered_dataset;
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::IntoOverwriteActive;
use crate::services::repo::traits::shared::RegisteredDatasetRepoTrait;

const UPSERT_BATCH_SIZE: usize = 500;

pub struct RegisteredDatasetPostgresRepo {
    db: DatabaseConnection,
}

impl RegisteredDatasetPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn upsert_chunk(
        conn: &impl ConnectionTrait,
        datasets: Vec<registered_dataset::Model>,
    ) -> Outcome<()> {
        let rows = datasets.into_iter().map(IntoOverwriteActive::into_active);
        registered_dataset::Entity::insert_many(rows)
            .on_conflict(
                OnConflict::column(registered_dataset::Column::Id)
                    .update_columns([
                        registered_dataset::Column::CatalogId,
                        registered_dataset::Column::Location,
                        registered_dataset::Column::Requirements,
                        registered_dataset::Column::OwnerUserId,
                        registered_dataset::Column::OwnerRole,
                        registered_dataset::Column::OwnerVisibility,
                        registered_dataset::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec_without_returning(conn)
            .await
            .map(|_| ())
            .map_err(|e| Errors::db("Unable to upsert the registered datasets", Some(Box::new(e))))
    }
}

#[async_trait]
impl RegisteredDatasetRepoTrait for RegisteredDatasetPostgresRepo {
    async fn replace_all(&self, datasets: Vec<registered_dataset::Model>) -> Outcome<()> {
        let fail = |e| Errors::db("Unable to replace the registered datasets", Some(Box::new(e)));
        let txn = self.db.begin().await.map_err(fail)?;
        registered_dataset::Entity::delete_many().exec(&txn).await.map_err(fail)?;
        if !datasets.is_empty() {
            let rows = datasets.into_iter().map(IntoOverwriteActive::into_active);
            registered_dataset::Entity::insert_many(rows).exec(&txn).await.map_err(fail)?;
        }
        txn.commit().await.map_err(fail)
    }

    async fn upsert_datasets(&self, datasets: Vec<registered_dataset::Model>) -> Outcome<()> {
        if datasets.is_empty() {
            return Ok(());
        }
        if datasets.len() <= UPSERT_BATCH_SIZE {
            return Self::upsert_chunk(&self.db, datasets).await;
        }
        let fail = |e| Errors::db("Unable to upsert the registered datasets", Some(Box::new(e)));
        let txn = self.db.begin().await.map_err(fail)?;
        let mut iter = datasets.into_iter();
        loop {
            let batch: Vec<_> = iter.by_ref().take(UPSERT_BATCH_SIZE).collect();
            if batch.is_empty() {
                break;
            }
            Self::upsert_chunk(&txn, batch).await?;
        }
        txn.commit().await.map_err(fail)
    }

    async fn delete_datasets(&self, ids: &[String]) -> Outcome<()> {
        if ids.is_empty() {
            return Ok(());
        }
        registered_dataset::Entity::delete_many()
            .filter(registered_dataset::Column::Id.is_in(ids.iter().cloned()))
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Errors::db("Unable to delete the registered datasets", Some(Box::new(e))))
    }

    async fn get_by_ids(&self, ids: &[String]) -> Outcome<Vec<registered_dataset::Model>> {
        registered_dataset::Entity::find()
            .filter(registered_dataset::Column::Id.is_in(ids.iter().cloned()))
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("Unable to get registered datasets", Some(Box::new(e))))
    }

    async fn get_all(&self) -> Outcome<Vec<registered_dataset::Model>> {
        registered_dataset::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("Unable to get registered datasets", Some(Box::new(e))))
    }
}
