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
use crate::errors::{ErrorLogTrait, Errors};

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

    async fn get_all(
        &self,
        limit: Option<u64>,
        offset: Option<u64>
    ) -> anyhow::Result<Vec<T::Model>> {
        let models = T::find()
            .limit(limit.unwrap_or(100000))
            .offset(offset.unwrap_or(0))
            .all(self.db())
            .await;
        let models = models.map_err(|e| Errors::db_tap(e.to_string()))?;

        Ok(models)
    }

    async fn get_by_id(&self, id: &str) -> anyhow::Result<T::Model> {
        let model = T::find_by_id(id).one(self.db()).await;

        let model = model
            .map_err(|e| Errors::db_tap(e.to_string()))?
            .ok_or_else(|| Errors::miss_tap(id))?;
        Ok(model)
    }

    async fn create(&self, model: U) -> anyhow::Result<T::Model> {
        let active_model: T::ActiveModel = model.to_active();
        let model: T::Model =
            active_model.insert(self.db()).await.map_err(|e| Errors::db_tap(e.to_string()))?;
        Ok(model)
    }

    async fn update(&self, model: T::Model) -> anyhow::Result<T::Model> {
        let active_model: T::ActiveModel = model.to_active();
        let new_model: T::Model =
            active_model.update(self.db()).await.map_err(|e| Errors::db_tap(e.to_string()))?;
        Ok(new_model)
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let model = self.get_by_id(id).await?;
        let active_model: T::ActiveModel = model.to_active();

        active_model.delete(self.db()).await.map_err(|e| Errors::db_tap(e.to_string()))?;
        Ok(())
    }

    async fn help_find(
        &self,
        to_find: Select<T>,
        resource: &str,
        id: &str
    ) -> anyhow::Result<T::Model> {
        let model =
            to_find.one(self.db()).await.map_err(|e| Errors::db_tap(e.to_string()))?.ok_or_else(
                || {
                    let error = Errors::missing_resource_new(
                        id,
                        &format!("Missing resource {} with id: {}", resource, id)
                    );
                    tracing::error!("{}", error.log());
                    error
                }
            )?;
        Ok(model)
    }
}
