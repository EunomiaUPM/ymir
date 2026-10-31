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

use crate::impl_serde_via_str;
use sea_orm::FromJsonQueryResult;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, FromJsonQueryResult)]
pub enum AccessTokenFlag {
    Bearer,
    Durable,
    Split,
    Other(String),
}

impl Display for AccessTokenFlag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AccessTokenFlag::Bearer => "bearer",
            AccessTokenFlag::Durable => "durable",
            AccessTokenFlag::Split => "split",
            AccessTokenFlag::Other(other) => other.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AccessTokenFlag {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "bearer" => Ok(AccessTokenFlag::Bearer),
            "durable" => Ok(AccessTokenFlag::Durable),
            "split" => Ok(AccessTokenFlag::Split),
            _ => Ok(AccessTokenFlag::Other(s.to_string())),
        }
    }
}

impl_serde_via_str!(AccessTokenFlag);
