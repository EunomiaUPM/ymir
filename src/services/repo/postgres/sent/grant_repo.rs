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
use sea_orm::sea_query::{Condition, Expr, Func};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::entities::sent::grant;
use crate::data::entities::sent::grant::Model;
use crate::errors::Outcome;
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::postgres::listing::KeysetPager;
use crate::services::repo::traits::sent::SentGrantRepoTrait;
use crate::types::listing::{GrantSort, ListPage, Listed, SentGrantListFilter};

pub struct SentGrantPostgresRepo {
    db: DatabaseConnection,
}

impl SentGrantPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for SentGrantPostgresRepo {
    type Entity = grant::Entity;
    type Plan = grant::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl SentGrantRepoTrait for SentGrantPostgresRepo {
    async fn find_page(
        &self,
        filter: &SentGrantListFilter,
        page: &ListPage<GrantSort>,
    ) -> Outcome<Listed<Model>> {
        let mut select = grant::Entity::find().filter(grant::Column::Kind.eq(filter.kind.clone()));
        // Own grants, or grants requested under a role strictly below the caller's.
        select = select.filter(
            Condition::any()
                .add(grant::Column::UserId.eq(filter.user_id.as_str()))
                .add(grant::Column::Role.like(KeysetPager::below(&filter.role))),
        );
        if let Some(id) = &filter.participant_id_contains {
            select = select.filter(grant::Column::ParticipantId.like(KeysetPager::contains(id)));
        }
        if let Some(nick) = &filter.nick_contains {
            select = select.filter(
                Expr::expr(Func::lower(Expr::col(grant::Column::ParticipantNick)))
                    .like(KeysetPager::contains(&nick.to_lowercase())),
            );
        }
        if let Some(status) = &filter.status {
            select = select.filter(grant::Column::Status.eq(status.clone()));
        }
        if let Some(after) = filter.created_after {
            select = select.filter(grant::Column::CreatedAt.gte(after));
        }
        if let Some(before) = filter.created_before {
            select = select.filter(grant::Column::CreatedAt.lte(before));
        }
        let timestamp = match page.sort {
            GrantSort::Created => Expr::col(grant::Column::CreatedAt).into(),
            // Open grants have no end yet; they sort by creation.
            GrantSort::Updated => Func::coalesce([
                Expr::col(grant::Column::EndedAt).into(),
                Expr::col(grant::Column::CreatedAt).into(),
            ])
            .into(),
        };
        KeysetPager::fetch(self.db(), select, timestamp, grant::Column::Id, page).await
    }
}
