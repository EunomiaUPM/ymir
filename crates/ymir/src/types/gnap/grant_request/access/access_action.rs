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

//! What a GNAP4DS `access` element asks to do with its resource.

use crate::impl_serde_via_str;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum AccessAction {
    /// Read the resource; what an element without `actions` asks for.
    #[default]
    View,
    /// Negotiate an offer of the resource and transfer under its agreement.
    Use,
    Other(String),
}

impl AccessAction {
    const VIEW: &'static str = "view";
    const USE: &'static str = "use";
}

impl Display for AccessAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AccessAction::View => Self::VIEW,
            AccessAction::Use => Self::USE,
            AccessAction::Other(other) => other.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AccessAction {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            Self::VIEW => AccessAction::View,
            Self::USE => AccessAction::Use,
            _ => AccessAction::Other(s.to_string()),
        })
    }
}

impl_serde_via_str!(AccessAction);
