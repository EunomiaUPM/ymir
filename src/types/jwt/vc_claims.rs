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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::types::vcs::doc::VcDocument;
use super::{VcJwtClaimsV1, VcJwtClaimsV2};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum VCJwtClaims {
    V1(VcJwtClaimsV1),
    V2(VcJwtClaimsV2),
}

impl VCJwtClaims {
    pub fn iss(&self) -> Option<&str> {
        match self {
            VCJwtClaims::V1(claims) => { claims.iss.as_deref() }
            VCJwtClaims::V2(claims) => { claims.iss.as_deref() }
        }
    }
    pub fn sub(&self) -> Option<&str> {
        match self {
            VCJwtClaims::V1(claims) => { claims.sub.as_deref() }
            VCJwtClaims::V2(claims) => { claims.sub.as_deref() }
        }
    }
    pub fn jti(&self) -> Option<&str> {
        match self {
            VCJwtClaims::V1(claims) => { claims.jti.as_deref() }
            VCJwtClaims::V2(claims) => { claims.jti.as_deref() }
        }
    }
    pub fn nbf(&self) -> Option<i64> {
        match self {
            VCJwtClaims::V1(claims) => { claims.nbf }
            VCJwtClaims::V2(claims) => { claims.nbf }
        }
    }
    pub fn exp(&self) -> Option<i64> {
        match self {
            VCJwtClaims::V1(claims) => { claims.exp }
            VCJwtClaims::V2(claims) => { claims.exp }
        }
    }
    pub fn iat(&self) -> Option<i64> {
        match self {
            VCJwtClaims::V1(claims) => { claims.iat }
            VCJwtClaims::V2(claims) => { claims.iat }
        }
    }



    pub fn vc_doc(&self) -> &VcDocument {
        match self {
            VCJwtClaims::V1(claims) => { &claims.vc }
            VCJwtClaims::V2(claims) => { &claims.vc }
        }
    }
}

