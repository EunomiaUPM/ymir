//  * Copyright (C) 2024 - Universidad Politécnica de Madrid - UPM
//  *
//  * This program is free software: you can redistribute it and/or modify
//  * it under the terms of the GNU General Public License as published by
//  * the Free Software Foundation, either version 3 of the License, or
//  * (at your option) any later version.
//  *
//  * This program is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  * GNU General Public License for more details.
//  *
//  * You should have received a copy of the GNU General Public License
//  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
//

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, PrimaryKeyTrait,
    QuerySelect, Select
};

use crate::data::IntoActiveSet;
use crate::errors::{Errors, Outcome};

#[async_trait]
pub trait BasicRepoTrait<T, U>
where
    T: EntityTrait + Sync + Send + 'static,
    T::Model: Send
        + Sync
        + Clone
        + IntoActiveModel<T::ActiveModel>
        + IntoActiveSet<T::ActiveModel>
        + 'static,
    T::ActiveModel: ActiveModelTrait<Entity = T> + Send + Sync + 'static,
    U: IntoActiveSet<T::ActiveModel> + Send + Sync + 'static,
    <T as EntityTrait>::PrimaryKey: PrimaryKeyTrait<ValueType = String>
{
    fn db(&self) -> &DatabaseConnection;

    async fn get_all(&self, limit: Option<u64>, offset: Option<u64>) -> Outcome<Vec<T::Model>> {
        let models = T::find()
            .limit(limit.unwrap_or(100000))
            .offset(offset.unwrap_or(0))
            .all(self.db())
            .await;
        let models = models
            .map_err(|e| Errors::db("Unable to get all models", Some(anyhow::Error::from(e))))?;

        Ok(models)
    }

    async fn get_by_id(&self, id: &str) -> Outcome<T::Model> {
        let model = T::find_by_id(id).one(self.db()).await;

        let model = model
            .map_err(|e| {
                Errors::db(
                    format!("Unable to get model with id: {}", id),
                    Some(anyhow::Error::from(e))
                )
            })?
            .ok_or_else(|| {
                Errors::missing_resource(id, format!("Unable to get model with id: {}", id), None)
            })?;
        Ok(model)
    }

    async fn create(&self, model: U) -> Outcome<T::Model> {
        let active_model: T::ActiveModel = model.to_active();
        let model: T::Model = active_model
            .insert(self.db())
            .await
            .map_err(|e| Errors::db("Unable to create model", Some(anyhow::Error::from(e))))?;
        Ok(model)
    }

    async fn update(&self, model: T::Model) -> Outcome<T::Model> {
        let active_model: T::ActiveModel = model.to_active();
        let new_model: T::Model = active_model
            .update(self.db())
            .await
            .map_err(|e| Errors::db("Unable to update model", Some(anyhow::Error::from(e))))?;
        Ok(new_model)
    }

    async fn delete(&self, id: &str) -> Outcome<()> {
        let model = self.get_by_id(id).await?;
        let active_model: T::ActiveModel = model.to_active();

        active_model
            .delete(self.db())
            .await
            .map_err(|e| Errors::db("Unable to delete model", Some(anyhow::Error::from(e))))?;
        Ok(())
    }

    async fn help_find(&self, to_find: Select<T>, resource: &str, id: &str) -> Outcome<T::Model> {
        let model = to_find
            .one(self.db())
            .await
            .map_err(|e| {
                Errors::db(
                    format!("Unable to find model with column '{}' with value {}", resource, id),
                    Some(anyhow::Error::from(e))
                )
            })?
            .ok_or_else(|| {
                Errors::missing_resource(
                    id,
                    format!("Unable to find model with column '{}' with value {}", resource, id),
                    None
                )
            })?;
        Ok(model)
    }
}
