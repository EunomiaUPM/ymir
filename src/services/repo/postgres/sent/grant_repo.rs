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
use chrono::{DateTime, Utc};
use sea_orm::sea_query::{Condition, Expr, Func};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Select};

use crate::data::entities::sent::grant;
use crate::data::entities::sent::grant::Model;
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::postgres::listing::KeysetPager;
use crate::services::repo::traits::sent::SentGrantRepoTrait;
use crate::types::gnap::GrantStatus;
use crate::types::gnap::grant_request::GrantKind;
use crate::types::listing::{
    GrantSort, ListPage, Listed, SentGrantListFilter, VcRequestListFilter,
};
use crate::types::oauth::{RoleTrait, UserInfo};
use crate::types::participants::Visibility;

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

impl SentGrantPostgresRepo {
    /// Grants `user` sees: its own, those requested under a role below its own, and any not
    /// `Private`; every grant for the root.
    fn visible_to(user: &UserInfo) -> Condition {
        if user.is_root() {
            return Condition::all();
        }
        Condition::any()
            .add(grant::Column::UserId.eq(user.id()))
            .add(grant::Column::Role.like(KeysetPager::below(user.role())))
            .add(grant::Column::Visibility.ne(Visibility::Private))
    }

    /// Applies the filters both listings share and fetches the page.
    async fn fetch_page(
        &self,
        mut select: Select<grant::Entity>,
        common: CommonFilter<'_>,
        page: &ListPage<GrantSort>,
    ) -> Outcome<Listed<Model>> {
        if let Some(id) = common.participant_id_contains {
            select = select.filter(grant::Column::ParticipantId.like(KeysetPager::contains(id)));
        }
        if let Some(nick) = common.nick_contains {
            select = select.filter(
                Expr::expr(Func::lower(Expr::col(grant::Column::ParticipantNick)))
                    .like(KeysetPager::contains(&nick.to_lowercase())),
            );
        }
        if let Some(status) = common.status {
            select = select.filter(grant::Column::Status.eq(status.clone()));
        }
        if let Some(after) = common.created_after {
            select = select.filter(grant::Column::CreatedAt.gte(after));
        }
        if let Some(before) = common.created_before {
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

/// Filters shared by the access-token and VC-request listings.
struct CommonFilter<'a> {
    participant_id_contains: Option<&'a String>,
    nick_contains: Option<&'a String>,
    status: Option<&'a GrantStatus>,
    created_after: Option<DateTime<Utc>>,
    created_before: Option<DateTime<Utc>>,
}

#[async_trait]
impl SentGrantRepoTrait for SentGrantPostgresRepo {
    async fn find_page(
        &self,
        filter: &SentGrantListFilter,
        page: &ListPage<GrantSort>,
    ) -> Outcome<Listed<Model>> {
        // Kind fixed here so this private listing never returns VC requests, and vice versa.
        let select = grant::Entity::find()
            .filter(grant::Column::Kind.eq(GrantKind::AccessToken))
            .filter(Self::visible_to(&filter.tenant));
        let common = CommonFilter {
            participant_id_contains: filter.participant_id_contains.as_ref(),
            nick_contains: filter.nick_contains.as_ref(),
            status: filter.status.as_ref(),
            created_after: filter.created_after,
            created_before: filter.created_before,
        };
        self.fetch_page(select, common, page).await
    }

    async fn find_vc_requests_page(
        &self,
        filter: &VcRequestListFilter,
        page: &ListPage<GrantSort>,
    ) -> Outcome<Listed<Model>> {
        let select = grant::Entity::find()
            .filter(grant::Column::Kind.eq(GrantKind::CredentialRequest))
            .filter(Self::visible_to(&filter.tenant));
        let common = CommonFilter {
            participant_id_contains: filter.participant_id_contains.as_ref(),
            nick_contains: filter.nick_contains.as_ref(),
            status: filter.status.as_ref(),
            created_after: filter.created_after,
            created_before: filter.created_before,
        };
        self.fetch_page(select, common, page).await
    }

    async fn get_active_access(
        &self,
        user_id: &str,
        participant_id: &str,
    ) -> Outcome<Option<Model>> {
        grant::Entity::find()
            .filter(grant::Column::UserId.eq(user_id))
            .filter(grant::Column::ParticipantId.eq(participant_id))
            .filter(grant::Column::Kind.eq(GrantKind::AccessToken))
            .filter(grant::Column::Status.eq(GrantStatus::Approved))
            .filter(grant::Column::Token.is_not_null())
            .order_by_desc(grant::Column::CreatedAt)
            .one(self.db())
            .await
            .map_err(|e| Errors::db("Unable to get active access grant", Some(Box::new(e))))
    }
}
