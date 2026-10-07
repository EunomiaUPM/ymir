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
use sea_orm::sea_query::{Condition, OnConflict};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::entities::shared::participant_relation::{self, ANONYMOUS_USER_ID, Model};
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::IntoOverwriteActive;
use crate::services::repo::postgres::listing::KeysetPager;
use crate::services::repo::traits::shared::ParticipantRelationRepoTrait;
use crate::types::oauth::{RoleTrait, UserInfo, UserTrait};
use crate::types::participants::Visibility;

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

    async fn get_visible_by_participant(
        &self,
        user: &UserInfo,
        participant_id: &str,
    ) -> Outcome<Vec<Model>> {
        let mut select = participant_relation::Entity::find()
            .filter(participant_relation::Column::ParticipantId.eq(participant_id));
        if !user.is_root() {
            select = select.filter(
                Condition::any()
                    .add(participant_relation::Column::UserId.eq(user.id()))
                    .add(participant_relation::Column::Role.like(KeysetPager::below(user.role())))
                    .add(
                        participant_relation::Column::Visibility.ne(Visibility::Private),
                    ),
            );
        }
        let relations = select
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("Unable to get participant relations", Some(Box::new(e))))?;
        Ok(relations
            .into_iter()
            .map(|mut relation| {
                let anonymous = relation.visibility == Visibility::Anonymous
                    && !user.reaches(&relation.user_id, &relation.role);
                if anonymous {
                    relation.user_id = ANONYMOUS_USER_ID.to_string();
                    relation.username = None;
                }
                relation
            })
            .collect())
    }

    async fn force_update(&self, relation: Model) -> Outcome<Model> {
        participant_relation::Entity::insert(relation.into_active())
            .on_conflict(
                OnConflict::columns([
                    participant_relation::Column::UserId,
                    participant_relation::Column::ParticipantId,
                ])
                .update_columns([
                    participant_relation::Column::Username,
                    participant_relation::Column::Role,
                    participant_relation::Column::Visibility,
                ])
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
