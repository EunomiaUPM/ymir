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

use crate::data::entities::received::grant as recv_grant;
use crate::data::entities::shared::participant::Plan;
use crate::data::entities::shared::{participant, participant_relation};
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::listing::KeysetPager;
use crate::services::repo::postgres::{BasicPostgresRepo, IntoOverwriteActive};
use crate::services::repo::traits::shared::ParticipantRepoTrait;
use crate::types::listing::{ListPage, Listed, ParticipantListFilter, ParticipantSort};
use crate::types::participants::{ParticipantType, ParticipantVisibility};
use async_trait::async_trait;
use sea_orm::sea_query::{Condition, Expr, Func, OnConflict, Query};
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
    type Plan = Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl ParticipantRepoTrait for ParticipantPostgresRepo {
    async fn get_batch(&self, ids: &[String]) -> Outcome<Vec<participant::Model>> {
        participant::Entity::find()
            .filter(participant::Column::ParticipantId.is_in(ids))
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("Unable to get participant batch", Some(Box::new(e))))
    }

    async fn find_page(
        &self,
        filter: &ParticipantListFilter,
        page: &ListPage<ParticipantSort>,
    ) -> Outcome<Listed<participant::Model>> {
        let mut select = participant::Entity::find();
        // (a) added by the caller, (b) shared by someone, (c) added under a role below the
        // caller's.
        let by_relation = Query::select()
            .column(participant_relation::Column::ParticipantId)
            .from(participant_relation::Entity)
            .cond_where(
                Condition::any()
                    .add(participant_relation::Column::UserId.eq(filter.tenant.user_id()))
                    .add(
                        participant_relation::Column::Visibility.ne(ParticipantVisibility::Private),
                    )
                    .add(
                        participant_relation::Column::Role
                            .like(KeysetPager::below(filter.tenant.role())),
                    ),
            )
            .to_owned();
        // (d) has a received grant under the caller's role or one below it.
        let by_recv_grant = Query::select()
            .column(recv_grant::Column::ParticipantId)
            .from(recv_grant::Entity)
            .cond_where(
                Condition::any()
                    .add(recv_grant::Column::Role.eq(filter.tenant.role().clone()))
                    .add(recv_grant::Column::Role.like(KeysetPager::below(filter.tenant.role()))),
            )
            .to_owned();
        select = select.filter(
            Condition::any()
                .add(participant::Column::ParticipantId.in_subquery(by_relation))
                .add(participant::Column::ParticipantId.in_subquery(by_recv_grant)),
        );
        if filter.participant_type != ParticipantType::All {
            select = select
                .filter(participant::Column::ParticipantType.eq(filter.participant_type.clone()));
        }
        if let Some(nick) = &filter.nick_contains {
            select = select.filter(
                Expr::expr(Func::lower(Expr::col(participant::Column::ParticipantNick)))
                    .like(KeysetPager::contains(&nick.to_lowercase())),
            );
        }
        if let Some(id) = &filter.id_contains {
            select =
                select.filter(participant::Column::ParticipantId.like(KeysetPager::contains(id)));
        }
        if let Some(after) = filter.saved_after {
            select = select.filter(participant::Column::SavedAt.gte(after));
        }
        if let Some(before) = filter.saved_before {
            select = select.filter(participant::Column::SavedAt.lte(before));
        }
        let timestamp = match page.sort {
            ParticipantSort::SavedAt => Expr::col(participant::Column::SavedAt).into(),
            ParticipantSort::LastInteraction => {
                Expr::col(participant::Column::LastInteraction).into()
            }
        };
        KeysetPager::fetch(
            &self.db,
            select,
            timestamp,
            participant::Column::ParticipantId,
            page,
        )
        .await
    }

    async fn force_update(&self, plan: participant::Plan) -> Outcome<participant::Model> {
        participant::Entity::insert(plan.into_active())
            .on_conflict(
                OnConflict::columns([participant::Column::ParticipantId])
                    .update_columns([
                        participant::Column::BaseUrl,
                        participant::Column::LastInteraction,
                        participant::Column::ParticipantNick,
                    ])
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| Errors::db("Unable to upsert participant", Some(Box::new(e))))
    }
}
