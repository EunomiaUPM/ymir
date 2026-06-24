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

use crate::errors::{BadFormat, Errors};
use sea_orm::{DeriveActiveEnum, EnumIter};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq, DeriveActiveEnum, EnumIter, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "participant_type")]
pub enum ParticipantType {
    #[sea_orm(string_value = "Agent")]
    Agent,
    #[sea_orm(string_value = "Authority")]
    Authority,
    #[sea_orm(string_value = "All")]
    All,
}

impl Display for ParticipantType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticipantType::Agent => {
                write!(f, "Agent")
            }
            ParticipantType::Authority => {
                write!(f, "Authority")
            }
            ParticipantType::All => {
                write!(f, "All")
            }
        }
    }
}

impl FromStr for ParticipantType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "agent" | "agents" => Ok(ParticipantType::Agent),
            "authority" | "authorities" => Ok(ParticipantType::Authority),
            "all" => Ok(ParticipantType::All),
            other => Err(Errors::format(
                BadFormat::Received,
                format!("invalid participant type {other}"),
                None,
            )),
        }
    }
}
