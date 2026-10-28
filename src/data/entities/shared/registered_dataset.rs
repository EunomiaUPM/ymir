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

//! A dataset the catalog publishes, registered with the AS (GNAP RS): what a grant can give, what
//! it asks for, and who owns it inside the connector.

use crate::services::repo::postgres::IntoOverwriteActive;
use crate::types::oauth::RolePath;
use crate::types::participants::Visibility;
use crate::types::vcs::VcType;
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "registered_datasets")]
pub struct Model {
    /// The dataset's id, as a granted `dataset` names it in `identifier`.
    #[sea_orm(primary_key)]
    pub id: String,
    pub catalog_id: String,
    /// The `<base>/catalog/datasets` it is published under.
    pub location: String,
    /// Credentials it asks of a peer beyond the base.
    #[sea_orm(column_type = "JsonBinary")]
    pub requirements: Vec<VcType>,
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
            requirements: ActiveValue::Set(self.requirements),
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
