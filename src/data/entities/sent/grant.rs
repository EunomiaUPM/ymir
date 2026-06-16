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


use chrono::{DateTime, Utc};
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::data::entities::IntoOverwriteActive;
use crate::types::gnap::grant_request::GrantKind;
use crate::types::gnap::GrantStatus;
use crate::types::vcs::VcTypeConfig;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sent_grants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,                                 // ID of request
    pub participant_id: String,                     // ID of participant to who which we do the request
    pub participant_nick: String,                   // Nick of participant
    pub grant_endpoint: String,
    pub kind: GrantKind,                            // Type of request, (token or vc)
    pub status: GrantStatus,
    pub token: Option<String>,
    pub vc_type_config: Option<VcTypeConfig>,
    pub vc_uri: Option<String>,
    pub as_assigned_id: Option<String>,
    pub auto: bool,                                 // If active, redeeming credentials or presented them is automatic
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,
    pub participant_id: String,
    pub participant_nick: String,
    pub vc_type_config: Option<VcTypeConfig>,
    pub grant_endpoint: String,
    pub kind: GrantKind,
    pub auto: Option<bool>,
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            participant_id: ActiveValue::Set(self.participant_id),
            participant_nick: ActiveValue::Set(self.participant_nick),
            grant_endpoint: ActiveValue::Set(self.grant_endpoint),
            kind: ActiveValue::Set(self.kind),
            auto: ActiveValue::Set(self.auto.unwrap_or(false)),
            status: ActiveValue::Set(GrantStatus::Processing),
            token: ActiveValue::Set(None),
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
            participant_id: ActiveValue::Set(self.participant_id),
            participant_nick: ActiveValue::Set(self.participant_nick),
            grant_endpoint: ActiveValue::Set(self.grant_endpoint),
            kind: ActiveValue::Set(self.kind),
            auto: ActiveValue::Set(self.auto),
            status: ActiveValue::Set(self.status),
            token: ActiveValue::Set(self.token),
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
