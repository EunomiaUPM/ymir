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

//! A dataset the catalog publishes, registered with the AS (GNAP RS): what a grant can give, the
//! offers that say what it asks for, and who owns it inside the connector.

use crate::services::repo::postgres::IntoOverwriteActive;
use crate::types::oauth::RolePath;
use crate::types::participants::Visibility;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue, FromJsonQueryResult};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An offer of a dataset: its ODRL as the catalog holds it, for the AS to read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct RegisteredOffer {
    pub id: String,
    pub odrl: Value,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "registered_datasets")]
pub struct Model {
    /// The dataset's id, as a granted `dataset` names it in `identifier`.
    #[sea_orm(primary_key)]
    pub id: String,
    pub catalog_id: String,
    /// The `<base>/catalog/datasets` it is published under.
    pub location: String,
    /// Its offers, or those of its catalog if it has none.
    #[sea_orm(column_type = "JsonBinary")]
    pub offers: Vec<RegisteredOffer>,
    pub owner_user_id: String,
    pub owner_role: RolePath,
    pub owner_visibility: Visibility,
    pub updated_at: DateTime<Utc>,
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            catalog_id: ActiveValue::Set(self.catalog_id),
            location: ActiveValue::Set(self.location),
            offers: ActiveValue::Set(self.offers),
            owner_user_id: ActiveValue::Set(self.owner_user_id),
            owner_role: ActiveValue::Set(self.owner_role),
            owner_visibility: ActiveValue::Set(self.owner_visibility),
            updated_at: ActiveValue::Set(self.updated_at),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
