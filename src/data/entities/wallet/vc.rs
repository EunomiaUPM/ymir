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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::services::repo::postgres::IntoOverwriteActive;
use crate::types::issuance::VcBody;
use crate::types::vcs::{VcFormat, VcType};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, DeriveEntityModel};
use serde::{Deserialize, Serialize};
use serde_json::Value;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
    #[sea_orm(table_name = "vcs")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: String,
        pub vc_body: VcBody,
        pub vc_type: VcType,
        pub vc_format: VcFormat,
        pub holder_did: String,
        pub issuer_did: String,
        pub parsed_document: Value,
        pub valid_until: Option<DateTime<Utc>>,
        pub added_on: DateTime<Utc>,
    }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub vc_body: VcBody,
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            vc_body: ActiveValue::Set(self.vc_body),
            vc_type: ActiveValue::Set(self.vc_type),
            vc_format: ActiveValue::Set(self.vc_format),
            holder_did: ActiveValue::Set(self.holder_did),
            issuer_did: ActiveValue::Set(self.issuer_did),
            parsed_document: ActiveValue::Set(self.parsed_document),
            valid_until: ActiveValue::Set(self.valid_until),
            added_on: ActiveValue::Set(self.added_on),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
