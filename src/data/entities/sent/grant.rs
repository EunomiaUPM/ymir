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

use crate::services::repo::postgres::IntoOverwriteActive;
use crate::types::gnap::GrantStatus;
use crate::types::gnap::grant_request::GrantKind;
use crate::data::entities::shared::participant_relation::ANONYMOUS_USER_ID;
use crate::types::oauth::{RolePath, UserInfo, UserTrait};
use crate::types::participants::Visibility;
use crate::types::vcs::VcTypeConfig;
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sent_grants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String, // ID of request
    pub role: RolePath,
    pub user_id: String,
    /// Login name of `user_id` when the grant was requested, to show who asked for it.
    pub username: Option<String>,
    pub participant_id: String, // ID of participant to who which we do the request
    pub participant_nick: String, // Nick of participant
    /// Visibility the peer's relation gets once the grant completes.
    pub visibility: Visibility,
    pub grant_endpoint: String,
    pub kind: GrantKind, // Type of request, (token or vc)
    pub status: GrantStatus,
    pub final_token: Option<String>,
    pub final_expires_at: Option<DateTime<Utc>>,
    pub managing_uri: Option<String>,
    pub managing_token: Option<String>,
    pub managing_expires_at: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "JsonBinary")]
    pub vc_type_config: Option<Vec<VcTypeConfig>>,
    pub vc_uri: Option<String>,
    pub as_assigned_id: Option<String>,
    pub auto: bool, // If active, redeeming credentials or presented them is automatic
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl Model {
    /// The grant as `user` may get it: whole if it reaches the grant (its author, a role above,
    /// or the root); otherwise without its secrets (the peer's tokens and managing URI, the
    /// credential offer URI) and, if `Anonymous`, without its author.
    pub fn seen_by(mut self, user: &UserInfo) -> Self {
        if user.reaches(&self.user_id, &self.role) {
            return self;
        }
        self.final_token = None;
        self.managing_uri = None;
        self.managing_token = None;
        self.vc_uri = None;
        if self.visibility == Visibility::Anonymous {
            self.user_id = ANONYMOUS_USER_ID.to_string();
            self.username = None;
        }
        self
    }
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,
    pub role: RolePath,
    pub user_id: String,
    pub username: Option<String>,
    pub participant_id: String,
    pub participant_nick: String,
    pub visibility: Visibility,
    pub vc_type_config: Option<Vec<VcTypeConfig>>,
    pub grant_endpoint: String,
    pub kind: GrantKind,
    pub auto: Option<bool>,
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            role: ActiveValue::Set(self.role),
            user_id: ActiveValue::Set(self.user_id),
            username: ActiveValue::Set(self.username),
            participant_id: ActiveValue::Set(self.participant_id),
            participant_nick: ActiveValue::Set(self.participant_nick),
            visibility: ActiveValue::Set(self.visibility),
            grant_endpoint: ActiveValue::Set(self.grant_endpoint),
            kind: ActiveValue::Set(self.kind),
            auto: ActiveValue::Set(self.auto.unwrap_or(false)),
            status: ActiveValue::Set(GrantStatus::Processing),
            final_token: ActiveValue::Set(None),
            final_expires_at: ActiveValue::Set(None),
            managing_uri: ActiveValue::Set(None),
            managing_token: ActiveValue::Set(None),
            managing_expires_at: ActiveValue::Set(None),
            vc_type_config: ActiveValue::Set(self.vc_type_config),
            vc_uri: ActiveValue::Set(None),
            as_assigned_id: ActiveValue::Set(None),
            created_at: ActiveValue::Set(Utc::now()),
            ended_at: ActiveValue::Set(None),
        }
    }
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            role: ActiveValue::Set(self.role),
            user_id: ActiveValue::Set(self.user_id),
            username: ActiveValue::Set(self.username),
            participant_id: ActiveValue::Set(self.participant_id),
            participant_nick: ActiveValue::Set(self.participant_nick),
            visibility: ActiveValue::Set(self.visibility),
            grant_endpoint: ActiveValue::Set(self.grant_endpoint),
            kind: ActiveValue::Set(self.kind),
            auto: ActiveValue::Set(self.auto),
            status: ActiveValue::Set(self.status),
            final_token: ActiveValue::Set(self.final_token),
            final_expires_at: ActiveValue::Set(self.final_expires_at),
            managing_uri: ActiveValue::Set(self.managing_uri),
            managing_token: ActiveValue::Set(self.managing_token),
            managing_expires_at: ActiveValue::Set(self.managing_expires_at),
            vc_type_config: ActiveValue::Set(self.vc_type_config),
            vc_uri: ActiveValue::Set(self.vc_uri),
            as_assigned_id: ActiveValue::Set(self.as_assigned_id),
            created_at: ActiveValue::Set(self.created_at),
            ended_at: ActiveValue::Set(self.ended_at),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
