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
use crate::types::participants::Visibility;
use crate::types::oauth::RolePath;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// `user_id` of the relations the gatekeeper creates for peers whose grant it approved: the
/// peer was brought in by verification, not by a person.
pub const VERIFICATION_USER_ID: &str = "verification";

/// `user_id` shown in place of the real one for an `Anonymous` relation the viewer does not
/// reach: someone added the participant, but not who.
pub const ANONYMOUS_USER_ID: &str = "anonymous";

/// A user added a participant, under which role, and how visible it left it to the rest of
/// the organisation. The participant's own data lives in `participants`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "participant_relations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub participant_id: String,
    /// Login name of `user_id` when it added the participant; never shown for `Anonymous`.
    pub username: Option<String>,
    /// Role the user had when adding it; roles above it see the participant even if `Private`.
    pub role: RolePath,
    pub visibility: Visibility,
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            user_id: ActiveValue::Set(self.user_id),
            participant_id: ActiveValue::Set(self.participant_id),
            username: ActiveValue::Set(self.username),
            role: ActiveValue::Set(self.role),
            visibility: ActiveValue::Set(self.visibility),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
