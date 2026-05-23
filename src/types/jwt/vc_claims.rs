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

use std::marker::PhantomData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::types::present::{Missing, Present};
use crate::types::vcs::doc::VcDocument;
use crate::types::vcs::W3cDataModelVersion;
use super::{VcJwtClaimsV1, VcJwtClaimsV2};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum VCJwtClaims {
    V1(VcJwtClaimsV1),
    V2(VcJwtClaimsV2),
}

pub struct VcJwtClaimsBuilder<VC> {
    w3c_data_model_version: W3cDataModelVersion,
    iss: Option<String>,
    sub: Option<String>,
    jti: Option<String>,
    nbf: Option<i64>,
    exp: Option<i64>,
    iat: Option<i64>,
    vc: Option<VcDocument>,
    _marker: PhantomData<VC>,
}

impl VcJwtClaimsBuilder<Missing> {
    pub fn new(w3c_data_model_version: &W3cDataModelVersion) -> Self {
        VcJwtClaimsBuilder {
            w3c_data_model_version: w3c_data_model_version.clone(),
            iss: None,
            sub: None,
            jti: None,
            nbf: None,
            exp: None,
            iat: None,
            vc: None,
            _marker: PhantomData,
        }
    }
}

impl<VC> VcJwtClaimsBuilder<VC> {
    pub fn vc(self, vc_document: VcDocument) -> VcJwtClaimsBuilder<Present> {
        VcJwtClaimsBuilder {
            w3c_data_model_version: self.w3c_data_model_version,
            iss: self.iss,
            sub: self.sub,
            jti: self.jti,
            nbf: self.nbf,
            exp: self.exp,
            iat: self.iat,
            vc: Some(vc_document),
            _marker: PhantomData,
        }
    }
    pub fn iss(mut self, iss: impl Into<String>) -> Self {
        self.iss = Some(iss.into());
        self
    }

    pub fn sub(mut self, sub: impl Into<String>) -> Self {
        self.sub = Some(sub.into());
        self
    }

    pub fn jti(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    pub fn nbf(mut self, nbf: DateTime<Utc>) -> Self {
        self.nbf = Some(nbf.timestamp());
        self
    }

    pub fn exp(mut self, exp: DateTime<Utc>) -> Self {
        self.exp = Some(exp.timestamp());
        self
    }

    pub fn iat(mut self, iat: DateTime<Utc>) -> Self {
        self.iat = Some(iat.timestamp());
        self
    }
}

impl VcJwtClaimsBuilder<Present> {
    pub fn build(self) -> VCJwtClaims {
        match self.w3c_data_model_version {
            W3cDataModelVersion::V1 => VCJwtClaims::V1(VcJwtClaimsV1 {
                iss: self.iss,
                sub: self.sub,
                jti: self.jti,
                nbf: self.nbf,
                exp: self.exp,
                iat: self.iat,
                vc: self.vc.unwrap(),
            }),

            W3cDataModelVersion::V2 => VCJwtClaims::V2(VcJwtClaimsV2 {
                iss: self.iss,
                sub: self.sub,
                jti: self.jti,
                nbf: self.nbf,
                exp: self.exp,
                iat: self.iat,
                vc: self.vc.unwrap(),
            }),
        }
    }
}