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
use crate::types::participants::ParticipantVisibility;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A user added a participant, and how visible it left it to the rest of the organisation.
/// The participant's own data lives in `participants`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "participant_relations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub participant_id: String,
    pub visibility: ParticipantVisibility,
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub user_id: String,
    pub participant_id: String,
    /// Defaults to `Public`.
    pub visibility: Option<ParticipantVisibility>,
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            user_id: ActiveValue::Set(self.user_id),
            participant_id: ActiveValue::Set(self.participant_id),
            visibility: ActiveValue::Set(self.visibility.unwrap_or_default()),
        }
    }
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            user_id: ActiveValue::Set(self.user_id),
            participant_id: ActiveValue::Set(self.participant_id),
            visibility: ActiveValue::Set(self.visibility),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
