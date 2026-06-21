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
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct BuildCtx {
    pub subject_name: String,
    pub holder_did: Option<String>,
    pub cert: Option<String>,
    pub vcs: Vec<String>,
    pub claims: Value,
}

impl BuildCtx {
    pub fn base(subject_name: impl Into<String>, cert: Option<String>) -> Self {
        Self {
            subject_name: subject_name.into(),
            holder_did: None,
            cert,
            vcs: Vec::new(),
            claims: Value::Null,
        }
    }

    pub fn holder_did(mut self, holder_did: impl Into<String>) -> Self {
        self.holder_did = Some(holder_did.into());
        self
    }
    pub fn cert(mut self, cert: impl Into<String>) -> Self {
        self.cert = Some(cert.into());
        self
    }
    pub fn vcs(mut self, vcs: Vec<String>) -> Self {
        self.vcs = vcs;
        self
    }
    pub fn push_vc(mut self, vc: impl Into<String>) -> Self {
        self.vcs.push(vc.into());
        self
    }
    pub fn claim(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let value = serde_json::to_value(value).unwrap_or(Value::Null);

        let map = self.claims.as_object_mut()
            .expect("claims must be initialized as a JSON object");

        map.insert(key.into(), value);
        self
    }
}