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
use crate::types::verification::VerificationStatus;
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sent_verifications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String, // REQUEST
    pub uri: String,                     // REQUEST
    pub scheme: String,                  // REQUEST
    pub response_type: String,           // REQUEST
    pub client_id: String,               // REQUEST
    pub response_mode: String,           // REQUEST
    pub pd_uri: String,                  // REQUEST
    pub client_id_scheme: String,        // REQUEST
    pub nonce: String,                   // REQUEST
    pub response_uri: String,            // REQUEST
    pub status: VerificationStatus,      // DEFAULT
    pub created_at: DateTime<Utc>,       // DEFAULT
    pub ended_at: Option<DateTime<Utc>>, // RESPONSE
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,               // REQUEST
    pub uri: String,              // REQUEST
    pub scheme: String,           // REQUEST
    pub response_type: String,    // REQUEST
    pub client_id: String,        // REQUEST
    pub response_mode: String,    // REQUEST
    pub pd_uri: String,           // REQUEST
    pub client_id_scheme: String, // REQUEST
    pub nonce: String,            // REQUEST
    pub response_uri: String,     // REQUEST
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            uri: ActiveValue::Set(self.uri),
            scheme: ActiveValue::Set(self.scheme),
            response_type: ActiveValue::Set(self.response_type),
            client_id: ActiveValue::Set(self.client_id),
            response_mode: ActiveValue::Set(self.response_mode),
            pd_uri: ActiveValue::Set(self.pd_uri),
            client_id_scheme: ActiveValue::Set(self.client_id_scheme),
            nonce: ActiveValue::Set(self.nonce),
            response_uri: ActiveValue::Set(self.response_uri),
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
            uri: ActiveValue::Set(self.uri),
            scheme: ActiveValue::Set(self.scheme),
            response_type: ActiveValue::Set(self.response_type),
            client_id: ActiveValue::Set(self.client_id),
            response_mode: ActiveValue::Set(self.response_mode),
            pd_uri: ActiveValue::Set(self.pd_uri),
            client_id_scheme: ActiveValue::Set(self.client_id_scheme),
            nonce: ActiveValue::Set(self.nonce),
            response_uri: ActiveValue::Set(self.response_uri),
            status: ActiveValue::Set(self.status),
            created_at: ActiveValue::Set(self.created_at),
            ended_at: ActiveValue::Set(self.ended_at),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
