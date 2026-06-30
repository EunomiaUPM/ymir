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

use crate::data::entities::shared::participant;
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::postgres::IntoOverwriteActive;
use crate::services::repo::traits::shared::ParticipantRepoTrait;
use crate::types::participants::ParticipantType;
use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct ParticipantPostgresRepo {
    db: DatabaseConnection,
}

impl ParticipantPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for ParticipantPostgresRepo {
    type Entity = participant::Entity;
    type Plan = participant::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl ParticipantRepoTrait for ParticipantPostgresRepo {
    async fn get_me(&self) -> Outcome<participant::Model> {
        let query = participant::Entity::find().filter(participant::Column::IsMe.eq(true));

        self.basic_filter(query, "is_me", "true").await
    }

    async fn filter_by_type(
        &self,
        participant_type: ParticipantType,
    ) -> Outcome<Vec<participant::Model>> {
        match participant_type {
            ParticipantType::All => { self.basic_get_all(None, None).await }
            filter => {
                participant::Entity::find()
                    .filter(participant::Column::ParticipantType.eq(filter))
                    .all(self.db())
                    .await
                    .map_err(|e| Errors::db("Unable to get participant by type", Some(Box::new(e))))
            }
        }
    }

    async fn get_by_token(&self, token: &str) -> Outcome<participant::Model> {
        let query = participant::Entity::find().filter(participant::Column::Token.eq(token));

        self.basic_filter(query, "token", token).await
    }

    async fn get_batch(&self, ids: &[String]) -> Outcome<Vec<participant::Model>> {
        let mates = participant::Entity::find()
            .filter(participant::Column::ParticipantId.is_in(ids))
            .all(self.db())
            .await
            .map_err(|e| Errors::db("Error forcing getting batch", Some(Box::new(e))))?;
        Ok(mates)
    }

    async fn force_update(&self, plan: participant::Plan) -> Outcome<participant::Model> {
        let active_mate = plan.into_active();
        participant::Entity::insert(active_mate)
            .on_conflict(
                OnConflict::column(participant::Column::ParticipantId)
                    .update_columns([
                        participant::Column::BaseUrl,
                        participant::Column::LastInteraction,
                        participant::Column::Token,
                        participant::Column::ParticipantNick,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(self.db())
            .await
            .map_err(|e| Errors::db("Error forcing creating mate", Some(Box::new(e))))
    }
}
