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

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use super::AccessType;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, FromJsonQueryResult)]
pub struct ResourceAccess {
    pub r#type: AccessType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datatypes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privileges: Option<Vec<String>>,
}

impl ResourceAccess {
    fn new(r#type: AccessType, locations: Option<Vec<String>>, identifier: Option<String>) -> Self {
        Self {
            r#type,
            actions: None,
            locations,
            datatypes: None,
            identifier,
            privileges: None,
        }
    }

    pub fn catalog(location: impl Into<String>) -> Self {
        Self::new(AccessType::Catalog, Some(vec![location.into()]), None)
    }

    pub fn dataset(identifier: impl Into<String>) -> Self {
        Self::new(AccessType::Dataset, None, Some(identifier.into()))
    }

    pub fn datasets_of(location: impl Into<String>) -> Self {
        Self::new(AccessType::Dataset, Some(vec![location.into()]), None)
    }

    pub fn callback(location: impl Into<String>) -> Self {
        Self::new(AccessType::Callback, Some(vec![location.into()]), None)
    }

    pub fn offer(location: impl Into<String>) -> Self {
        Self::new(AccessType::Offer, Some(vec![location.into()]), None)
    }
}
