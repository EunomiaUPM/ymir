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
use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::present::{Missing, Present};

#[derive(Debug, Serialize, Deserialize)]
pub struct GaiaVP {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub r#type: String,
    #[serde(rename = "verifiableCredential")]
    pub verifiable_credential: Vec<VcInsideGaiaVP>,
    pub issuer: String,
    #[serde(rename = "validFrom", skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(rename = "validUntil", skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>
}

pub struct GaiaVPBuilder<R, S, T, U> {
    pub context: Vec<String>,
    pub r#type: Option<String>,
    pub verifiable_credential: Vec<VcInsideGaiaVP>,
    pub issuer: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    _marker: PhantomData<(R, S, T, U)>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VcInsideGaiaVP {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub r#type: String
}

pub struct VcInsideGaiaVPBuilder<T, S> {
    pub context: Vec<String>,
    pub id: Option<String>,
    pub r#type: String,
    _marker: PhantomData<(T, S)>
}

impl Default for VcInsideGaiaVPBuilder<Missing, Missing> {
    fn default() -> Self {
        Self {
            context: vec![],
            id: None,
            r#type: "VerifiablePresentation".to_string(),
            _marker: PhantomData
        }
    }
}

impl<T, R> VcInsideGaiaVPBuilder<T, R> {
    pub fn context(self, context: Vec<String>) -> VcInsideGaiaVPBuilder<Present, T> {
        VcInsideGaiaVPBuilder { context, id: self.id, r#type: self.r#type, _marker: PhantomData }
    }
    pub fn id(self, id: String) -> VcInsideGaiaVPBuilder<T, Present> {
        VcInsideGaiaVPBuilder {
            context: self.context,
            id: Some(id),
            r#type: self.r#type,
            _marker: PhantomData
        }
    }
}

impl VcInsideGaiaVPBuilder<Present, Present> {
    pub fn build(self) -> VcInsideGaiaVP {
        VcInsideGaiaVP { context: self.context, id: self.id.unwrap(), r#type: self.r#type }
    }
}
