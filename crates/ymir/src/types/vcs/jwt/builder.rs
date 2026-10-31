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

use std::marker::PhantomData;

use chrono::{DateTime, Utc};

use super::{VcPayload, VcPayloadV1};
use crate::types::jwt::JwtClaims;
use crate::types::vcs::W3cDataModelVersion;
use crate::types::vcs::doc::VcDocument;
use crate::utils::{Missing, Present};

/// Builds a VC's JWT claims; it does not compile without the document (`VC`).
pub struct VcJwtClaimsBuilder<VC> {
    w3c_data_model_version: W3cDataModelVersion,
    claims: JwtClaims<()>,
    vc: Option<VcDocument>,
    _marker: PhantomData<VC>,
}

impl VcJwtClaimsBuilder<Missing> {
    pub fn new(w3c_data_model_version: W3cDataModelVersion) -> Self {
        VcJwtClaimsBuilder {
            w3c_data_model_version,
            claims: JwtClaims::default(),
            vc: None,
            _marker: PhantomData,
        }
    }
}

impl<VC> VcJwtClaimsBuilder<VC> {
    pub fn vc(self, vc_document: VcDocument) -> VcJwtClaimsBuilder<Present> {
        VcJwtClaimsBuilder {
            w3c_data_model_version: self.w3c_data_model_version,
            claims: self.claims,
            vc: Some(vc_document),
            _marker: PhantomData,
        }
    }

    pub fn iss(mut self, iss: impl Into<String>) -> Self {
        self.claims = self.claims.with_iss(iss);
        self
    }

    pub fn sub(mut self, sub: impl Into<String>) -> Self {
        self.claims = self.claims.with_sub(sub);
        self
    }

    pub fn jti(mut self, jti: impl Into<String>) -> Self {
        self.claims = self.claims.with_jti(jti);
        self
    }

    pub fn nbf(mut self, nbf: DateTime<Utc>) -> Self {
        self.claims = self.claims.with_nbf(nbf);
        self
    }

    pub fn exp(mut self, exp: DateTime<Utc>) -> Self {
        self.claims = self.claims.with_exp(exp);
        self
    }

    pub fn iat(mut self, iat: DateTime<Utc>) -> Self {
        self.claims = self.claims.with_iat(iat);
        self
    }
}

impl VcJwtClaimsBuilder<Present> {
    pub fn build(self) -> JwtClaims<VcPayload> {
        let vc = self.vc.unwrap();
        let body = match self.w3c_data_model_version {
            W3cDataModelVersion::V1 => VcPayload::V1(VcPayloadV1 { vc }),
            W3cDataModelVersion::V2 => VcPayload::V2(vc),
        };
        self.claims.with_body(body)
    }
}
