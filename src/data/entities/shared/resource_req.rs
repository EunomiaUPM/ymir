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
use crate::types::gnap::grant_request::access::{AccessTokenFlag, AccessType};
use crate::types::gnap::grant_request::interact::InteractAction;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "resources_reqs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String, // REQUEST
    pub r#type: AccessType,                  // REQUEST
    #[sea_orm(column_type = "JsonBinary")]
    pub actions: Vec<InteractAction>,        // REQUEST
    pub locations: Option<Vec<String>>,      // REQUEST
    pub datatypes: Option<Vec<String>>,      // REQUEST
    pub identifier: Option<String>,          // REQUEST
    pub privileges: Option<Vec<String>>,     // REQUEST
    pub label: Option<String>,               // REQUEST
    #[sea_orm(column_type = "JsonBinary")]
    pub flags: Option<Vec<AccessTokenFlag>>, // REQUEST
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            r#type: ActiveValue::Set(self.r#type),
            actions: ActiveValue::Set(self.actions),
            locations: ActiveValue::Set(self.locations),
            datatypes: ActiveValue::Set(self.datatypes),
            identifier: ActiveValue::Set(self.identifier),
            privileges: ActiveValue::Set(self.privileges),
            label: ActiveValue::Set(self.label),
            flags: ActiveValue::Set(self.flags),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
