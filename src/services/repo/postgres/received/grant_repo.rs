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

use crate::types::participants::Visibility;
use async_trait::async_trait;
use sea_orm::sea_query::{Condition, Expr, Func};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::data::entities::received::grant;
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::postgres::listing::KeysetPager;
use crate::services::repo::traits::received::RecvGrantRepoTrait;
use crate::types::gnap::GrantStatus;
use crate::types::gnap::grant_request::GrantKind;
use crate::types::listing::{GrantSort, ListPage, Listed, RecvGrantListFilter};

pub struct RecvGrantPostgresRepo {
    db: DatabaseConnection,
}

impl RecvGrantPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for RecvGrantPostgresRepo {
    type Entity = grant::Entity;
    type Plan = grant::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl RecvGrantRepoTrait for RecvGrantPostgresRepo {
    async fn find_page(
        &self,
        filter: &RecvGrantListFilter,
        page: &ListPage<GrantSort>,
    ) -> Outcome<Listed<grant::Model>> {
        let mut select = grant::Entity::find().filter(grant::Column::Kind.eq(filter.kind.clone()));
        // Grants assigned to the caller's role or to a role below it, and any public one.
        select = select.filter(
            Condition::any()
                .add(grant::Column::Role.eq(filter.role.clone()))
                .add(grant::Column::Role.like(KeysetPager::below(&filter.role)))
                .add(grant::Column::Visibility.eq(Visibility::Public)),
        );
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

    async fn get_approved_by_token(&self, token: &str) -> Outcome<grant::Model> {
        grant::Entity::find()
            .filter(grant::Column::Token.eq(token))
            .filter(grant::Column::Kind.eq(GrantKind::AccessToken))
            .filter(grant::Column::Status.eq(GrantStatus::Approved))
            .one(self.db())
            .await
            .map_err(|e| Errors::db("Unable to get grant by token", Some(Box::new(e))))?
            // The token itself stays out of the error, which ends up in the logs.
            .ok_or_else(|| Errors::missing_resource("token", "no approved grant for this token", None))
    }
}
