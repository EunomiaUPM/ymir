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
use crate::types::vcs::VcTypeConfig;
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "recv_grants")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String, // REQUEST
    pub participant_nick: String, // REQUEST
    pub kind: GrantKind,
    pub token: Option<String>, // COMPLETION
    pub vc_type_config: Option<Vec<VcTypeConfig>>,
    pub status: GrantStatus,             // DEFAULT
    pub created_at: DateTime<Utc>,       // DEFAULT
    pub ended_at: Option<DateTime<Utc>>, // COMPLETION
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,
    pub participant_nick: String,
    pub vc_type_config: Option<Vec<VcTypeConfig>>,
    pub kind: GrantKind,
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            participant_nick: ActiveValue::Set(self.participant_nick),
            kind: ActiveValue::Set(self.kind),
            token: ActiveValue::Set(None),
            vc_type_config: ActiveValue::Set(None),
            status: ActiveValue::Set(GrantStatus::Pending),
            created_at: ActiveValue::Set(Utc::now()),
            ended_at: ActiveValue::Set(None),
        }
    }
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            participant_nick: ActiveValue::Set(self.participant_nick),
            kind: ActiveValue::Set(self.kind),
            token: ActiveValue::Set(self.token),
            vc_type_config: ActiveValue::Set(self.vc_type_config),
            status: ActiveValue::Set(self.status),
            created_at: ActiveValue::Set(self.created_at),
            ended_at: ActiveValue::Set(self.ended_at),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
