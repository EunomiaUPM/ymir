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
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AccessType {
    Catalog,
    Dataset,
    Callback,
    Offer,
    Other(String),
}

impl AccessType {
    const CATALOG: &'static str = "https://w3id.org/gnap4ds/catalog";
    const DATASET: &'static str = "https://w3id.org/gnap4ds/dataset";
    const CALLBACK: &'static str = "https://w3id.org/gnap4ds/callback";
    const OFFER: &'static str = "https://w3id.org/gnap4ds/offer";
}

impl Display for AccessType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AccessType::Catalog => Self::CATALOG,
            AccessType::Dataset => Self::DATASET,
            AccessType::Callback => Self::CALLBACK,
            AccessType::Offer => Self::OFFER,
            AccessType::Other(other) => other.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for AccessType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            Self::CATALOG => AccessType::Catalog,
            Self::DATASET => AccessType::Dataset,
            Self::CALLBACK => AccessType::Callback,
            Self::OFFER => AccessType::Offer,
            _ => AccessType::Other(s.to_string()),
        })
    }
}

impl_serde_via_str!(AccessType);
