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

use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, DeriveActiveEnum, EnumIter, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(16))")]
pub enum VerificationStatus {
    #[sea_orm(string_value = "Pending")]
    Pending,
    #[sea_orm(string_value = "Verified")]
    Verified,
    #[sea_orm(string_value = "Failed")]
    Failed,
}
