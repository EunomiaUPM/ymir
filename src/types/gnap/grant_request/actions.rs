/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::fmt::Display;
use std::str::FromStr;

use crate::errors::Errors;

#[derive(Clone)]
pub enum InteractActions {
    Talk,
    RequestVc
}

impl Display for InteractActions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InteractActions::Talk => write!(f, "talk"),
            InteractActions::RequestVc => write!(f, "request-vc")
        }
    }
}

impl FromStr for InteractActions {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "talk" => Ok(InteractActions::Talk),
            "request-vc" => Ok(InteractActions::RequestVc),
            actions => Err(Errors::parse(
                format!("Unable to parse the requested action {}", actions),
                None
            ))
        }
    }
}
