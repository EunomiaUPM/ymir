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

#[derive(PartialEq, Eq, Debug, Clone, FromJsonQueryResult)]
pub enum HashMethod {
    Sha256,
    Sha384,
    Sha512,
    Other(String),
}

impl Display for HashMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HashMethod::Sha256 => "sha-256",
            HashMethod::Sha384 => "sha-384",
            HashMethod::Sha512 => "sha-512",
            HashMethod::Other(other) => other.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for HashMethod {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "sha-256" | "sha256" => Ok(HashMethod::Sha256),
            "sha-384" | "sha384" => Ok(HashMethod::Sha256),
            "sha-512" | "sha512" => Ok(HashMethod::Sha256),
            _ => Ok(HashMethod::Other(s.to_string())),
        }
    }
}

impl_serde_via_str!(HashMethod);
