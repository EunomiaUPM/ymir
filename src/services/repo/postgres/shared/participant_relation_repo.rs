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
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::entities::shared::participant_relation::{self, Model, Plan};
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::IntoOverwriteActive;
use crate::services::repo::traits::shared::ParticipantRelationRepoTrait;

pub struct ParticipantRelationPostgresRepo {
    db: DatabaseConnection,
}

impl ParticipantRelationPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ParticipantRelationRepoTrait for ParticipantRelationPostgresRepo {
    async fn get(&self, user_id: &str, participant_id: &str) -> Outcome<Model> {
        participant_relation::Entity::find_by_id((user_id.to_string(), participant_id.to_string()))
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("Unable to get participant relation", Some(Box::new(e))))?
            .ok_or_else(|| {
                Errors::missing_resource(
                    participant_id,
                    format!("User {user_id} has no relation with participant {participant_id}"),
                    None,
                )
            })
    }

    async fn get_by_participant(&self, participant_id: &str) -> Outcome<Vec<Model>> {
        participant_relation::Entity::find()
            .filter(participant_relation::Column::ParticipantId.eq(participant_id))
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("Unable to get participant relations", Some(Box::new(e))))
    }

    async fn upsert(&self, plan: Plan) -> Outcome<Model> {
        participant_relation::Entity::insert(plan.into_active())
            .on_conflict(
                OnConflict::columns([
                    participant_relation::Column::UserId,
                    participant_relation::Column::ParticipantId,
                ])
                .update_column(participant_relation::Column::Visibility)
                .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| Errors::db("Unable to upsert participant relation", Some(Box::new(e))))
    }

    async fn delete(&self, user_id: &str, participant_id: &str) -> Outcome<()> {
        participant_relation::Entity::delete_by_id((
            user_id.to_string(),
            participant_id.to_string(),
        ))
        .exec(&self.db)
        .await
        .map_err(|e| Errors::db("Unable to delete participant relation", Some(Box::new(e))))?;
        Ok(())
    }
}
