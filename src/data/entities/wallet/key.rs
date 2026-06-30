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
use crate::types::keys::{Crv, Kty};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, DeriveEntityModel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub alias: String,
    pub kty: Kty,
    pub crv: Option<Crv>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub alias: String,
    pub pem: String,
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            alias: ActiveValue::Set(self.alias),
            kty: ActiveValue::Set(self.kty),
            crv: ActiveValue::Set(self.crv),
            created_at: ActiveValue::Set(self.created_at),
        }
    }
}

// impl TryInto<PrivateKey> for Model {
//     type Error = Errors;
//
//     fn try_into(self) -> Result<PrivateKey, Self::Error> {
//         PrivateKey::from_safe_pem(&self.pem, &self.kty, self.crv.as_ref())
//     }
// }
//
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
