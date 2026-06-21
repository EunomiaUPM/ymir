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

use rand::Rng;
use rand::distributions::Alphanumeric;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::data::entities::IntoOverwriteActive;
use crate::types::vcs::VcType;
use crate::types::verifying::VerificationStatus;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "recv_verification")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String, // REQUEST
    pub state: String,                     // RANDOM
    pub nonce: String,                     // RANDOM
    pub vc_type: Vec<VcType>,              // REQUEST
    pub audience: String,                  // SEMI-RANDOM
    pub holder: Option<String>,            // RESPONSE
    pub vpt: Option<String>,               // RESPONSE
    pub vcs: Vec<String>,               // RESPONSE
    pub status: VerificationStatus,              // DEFAULT
    pub created_at: DateTime<Utc>,          // DEFAULT
    pub ended_at: Option<DateTime<Utc>>,    // RESPONSE
    // pub requirements: Value, TODO
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,           // REQUEST
    pub audience: String,     // SEMI-RANDOM
    pub vc_type: Vec<VcType>, // REQUEST
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        let state: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();
        let audience = format!("{}/{}", self.audience, &state);
        ActiveModel {
            id: ActiveValue::Set(self.id),
            state: ActiveValue::Set(state),
            nonce: ActiveValue::Set(nonce),
            vc_type: ActiveValue::Set(self.vc_type),
            audience: ActiveValue::Set(audience),
            holder: ActiveValue::Set(None),
            vpt: ActiveValue::Set(None),
            vcs: ActiveValue::Set(Vec::new()),
            status: ActiveValue::Set(VerificationStatus::Pending),
            created_at: ActiveValue::Set(Utc::now()),
            ended_at: ActiveValue::Set(None),
        }
    }
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            state: ActiveValue::Set(self.state),
            nonce: ActiveValue::Set(self.nonce),
            vc_type: ActiveValue::Set(self.vc_type),
            audience: ActiveValue::Set(self.audience),
            holder: ActiveValue::Set(self.holder),
            vpt: ActiveValue::Set(self.vpt),
            vcs: ActiveValue::Set(self.vcs),
            status: ActiveValue::Set(self.status),
            created_at: ActiveValue::Set(self.created_at),
            ended_at: ActiveValue::Set(self.ended_at),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
