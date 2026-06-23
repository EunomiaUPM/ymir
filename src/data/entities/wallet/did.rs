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
use crate::types::dids::{DidBuilder, DidDocument, DidService, DidType};
use crate::types::wallet::KeyRef;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, DeriveEntityModel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "dids")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub did: String,
    pub alias: String,
    pub r#default: bool,
    pub r#type: DidType,
    pub keys: Vec<KeyRef>,
    pub default_key: KeyRef,
    pub did_document: DidDocument,
    pub service: Option<Vec<DidService>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub alias: String,
    pub builder: DidBuilder,
    pub keys: Vec<String>,
    pub service: Option<Vec<DidService>>,
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            did: ActiveValue::Set(self.did),
            alias: ActiveValue::Set(self.alias),
            r#default: ActiveValue::Set(self.r#default),
            r#type: ActiveValue::Set(self.r#type),
            keys: ActiveValue::Set(self.keys),
            default_key: ActiveValue::Set(self.default_key),
            did_document: ActiveValue::Set(self.did_document),
            service: ActiveValue::Set(self.service),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
