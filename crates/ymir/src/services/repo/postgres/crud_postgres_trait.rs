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

use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::IntoOverwriteActive;
use crate::services::repo::traits::CrudRepoTrait;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PrimaryKeyTrait, QuerySelect, Select,
};

/// Structural Mixin for automated Sea-ORM Postgres CRUD execution.
///
/// Any repository containing a [`DatabaseConnection`] can implement this trait
/// to automatically qualify for a blanket [`CrudRepoTrait`] injection, minimizing boilerplate.
#[async_trait]
pub trait BasicPostgresRepo: Send + Sync + 'static
where
    <Self::Entity as EntityTrait>::Model: IntoOverwriteActive<<Self::Entity as EntityTrait>::ActiveModel>
        + IntoActiveModel<<Self::Entity as EntityTrait>::ActiveModel>
        + Send
        + Sync
        + Clone
        + 'static,
    <Self::Entity as EntityTrait>::ActiveModel:
        ActiveModelTrait<Entity = Self::Entity> + ActiveModelBehavior + Send + Sync + 'static,
    <Self::Entity as EntityTrait>::PrimaryKey: PrimaryKeyTrait<ValueType = String>,
{
    /// Associated Sea-ORM operational entity declaration.
    type Entity: EntityTrait + Send + Sync + 'static;

    /// Target Plan architecture used for insertions.
    type Plan: IntoOverwriteActive<<Self::Entity as EntityTrait>::ActiveModel>
        + Send
        + Sync
        + 'static;

    /// Exposes the inner active database reference pool.
    fn db(&self) -> &DatabaseConnection;

    async fn basic_get_all(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Outcome<Vec<<Self::Entity as EntityTrait>::Model>> {
        Self::Entity::find()
            .limit(limit.unwrap_or(100_000))
            .offset(offset.unwrap_or(0))
            .all(self.db())
            .await
            .map_err(|e| Errors::db("Unable to get all models", Some(Box::new(e))))
    }

    async fn basic_get_by_id(&self, id: &str) -> Outcome<<Self::Entity as EntityTrait>::Model> {
        Self::Entity::find_by_id(id.to_string())
            .one(self.db())
            .await
            .map_err(|e| {
                Errors::db(
                    format!("Unable to get model with id: {}", id),
                    Some(Box::new(e)),
                )
            })?
            .ok_or_else(|| Errors::missing_resource(id, format!("Model not found: {}", id), None))
    }

    async fn basic_create(
        &self,
        plan: Self::Plan,
    ) -> Outcome<<Self::Entity as EntityTrait>::Model> {
        let am = plan.into_active();
        am.insert(self.db())
            .await
            .map_err(|e| Errors::db("Unable to create model", Some(Box::new(e))))
    }

    async fn basic_update(
        &self,
        model: <Self::Entity as EntityTrait>::Model,
    ) -> Outcome<<Self::Entity as EntityTrait>::Model> {
        let am = model.into_active();
        am.update(self.db())
            .await
            .map_err(|e| Errors::db("Unable to update model", Some(Box::new(e))))
    }

    async fn basic_delete(&self, id: &str) -> Outcome<()> {
        Self::Entity::delete_by_id(id.to_string())
            .exec(self.db())
            .await
            .map_err(|e| Errors::db(format!("delete {} failed", id), Some(Box::new(e))))?;
        Ok(())
    }
    async fn basic_filter(
        &self,
        to_find: Select<Self::Entity>,
        resource: &str,
        id: &str,
    ) -> Outcome<<Self::Entity as EntityTrait>::Model> {
        to_find
            .one(self.db())
            .await
            .map_err(|e| {
                Errors::db(
                    format!(
                        "Unable to find model with column '{}' with value {}",
                        resource, id
                    ),
                    Some(Box::new(e)),
                )
            })?
            .ok_or_else(|| {
                Errors::missing_resource(
                    id,
                    format!(
                        "Unable to find model with column '{}' with value {}",
                        resource, id
                    ),
                    None,
                )
            })
    }
}

// ========================================= BLANKET IMPL ==========================================
#[async_trait]
impl<R> CrudRepoTrait<<R::Entity as EntityTrait>::Model, R::Plan> for R
where
    R: BasicPostgresRepo,
    <R::Entity as EntityTrait>::Model: IntoOverwriteActive<<R::Entity as EntityTrait>::ActiveModel>
        + IntoActiveModel<<R::Entity as EntityTrait>::ActiveModel>
        + Send
        + Sync
        + Clone
        + 'static,
    <R::Entity as EntityTrait>::ActiveModel:
        ActiveModelTrait<Entity = R::Entity> + ActiveModelBehavior + Send + Sync + 'static,
    <R::Entity as EntityTrait>::PrimaryKey: PrimaryKeyTrait<ValueType = String>,
{
    async fn get_all(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Outcome<Vec<<R::Entity as EntityTrait>::Model>> {
        self.basic_get_all(limit, offset).await
    }

    async fn get_by_id(&self, id: &str) -> Outcome<<R::Entity as EntityTrait>::Model> {
        self.basic_get_by_id(id).await
    }

    async fn create(&self, plan: R::Plan) -> Outcome<<R::Entity as EntityTrait>::Model> {
        self.basic_create(plan).await
    }

    async fn update(
        &self,
        model: <R::Entity as EntityTrait>::Model,
    ) -> Outcome<<R::Entity as EntityTrait>::Model> {
        self.basic_update(model).await
    }

    async fn delete(&self, id: &str) -> Outcome<()> {
        self.basic_delete(id).await
    }
}
