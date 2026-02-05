/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use chrono;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::Serialize;

use super::super::IntoActiveSet;

#[derive(Serialize, Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "vc_request")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String, // REQUEST
    pub participant_slug: String,                // REQUEST
    pub vc_type: String,                         // REQUEST
    pub cert: Option<String>,                    // REQUEST
    pub interact_method: Vec<String>,            // REQUEST
    pub vc_uri: Option<String>,                  // RESPONSE
    pub status: String,                          // DEFAULT
    pub is_vc_issued: bool,                      // COMPLETION
    pub created_at: chrono::NaiveDateTime,       // DEFAULT
    pub ended_at: Option<chrono::NaiveDateTime>, // COMPLETION
}

#[derive(Clone, Debug)]
pub struct NewModel {
    pub id: String,                   // REQUEST
    pub participant_slug: String,     // REQUEST
    pub vc_type: String,              // REQUEST
    pub interact_method: Vec<String>, // REQUEST
    pub cert: Option<String>,
}

impl IntoActiveSet<ActiveModel> for NewModel {
    fn to_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            participant_slug: ActiveValue::Set(self.participant_slug),
            vc_type: ActiveValue::Set(self.vc_type),
            cert: ActiveValue::Set(self.cert),
            interact_method: ActiveValue::Set(self.interact_method),
            vc_uri: ActiveValue::Set(None),
            status: ActiveValue::Set("Pending".to_string()),
            is_vc_issued: ActiveValue::Set(false),
            created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            ended_at: ActiveValue::Set(None),
        }
    }
}

impl IntoActiveSet<ActiveModel> for Model {
    fn to_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            participant_slug: ActiveValue::Set(self.participant_slug),
            vc_type: ActiveValue::Set(self.vc_type),
            cert: ActiveValue::Set(self.cert),
            interact_method: ActiveValue::Set(self.interact_method),
            vc_uri: ActiveValue::Set(self.vc_uri),
            status: ActiveValue::Set(self.status),
            is_vc_issued: ActiveValue::Set(self.is_vc_issued),
            created_at: ActiveValue::Set(self.created_at),
            ended_at: ActiveValue::Set(self.ended_at),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
