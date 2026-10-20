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

use serde::{Deserialize, Serialize};

use crate::types::crypto::{HasProofPurpose, ProofPurpose};
use crate::types::jwt::{HasJwtType, JwtClaims, JwtType};
use crate::types::vcs::W3cDataModelVersion;
use crate::types::vps::VpDocument;

/// The claims of a VP secured as a JWT; `aud` and `nonce` go in the registered claims.

/// What a VP's JWT carries besides the registered claims, per data model version.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum VpPayload {
    V1(VpPayloadV1),
    V2(VpDocument),
}

/// VC DM 1.1 §6.3.1: the presentation goes in the `vp` claim.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VpPayloadV1 {
    pub vp: VpDocument,
}

impl VpPayload {
    pub fn version(&self) -> W3cDataModelVersion {
        match self {
            VpPayload::V1(_) => W3cDataModelVersion::V1,
            VpPayload::V2(_) => W3cDataModelVersion::V2,
        }
    }
    pub fn vp_doc(&self) -> &VpDocument {
        match self {
            VpPayload::V1(v1) => &v1.vp,
            VpPayload::V2(doc) => doc,
        }
    }
}

impl JwtClaims<VpPayload> {
    pub fn vp_doc(&self) -> &VpDocument {
        self.body().vp_doc()
    }
}

impl HasProofPurpose for VpPayload {
    const PURPOSE: ProofPurpose = ProofPurpose::Authentication;
}

/// VC DM 1.1: `JWT`; VC-JOSE-COSE (VC DM 2.0): `vp+jwt` + `cty: vp`. Also reads Gaia-X 24.07's
/// `vp+ld+jwt`.
impl HasJwtType for VpPayload {
    const TYP: JwtType = JwtType::Vp;
    const CTY: Option<&'static str> = Some("vp");

    fn jwt_type(&self) -> JwtType {
        match self {
            VpPayload::V1(_) => JwtType::Jwt,
            VpPayload::V2(_) => JwtType::Vp,
        }
    }

    fn content_type(&self) -> Option<&'static str> {
        match self {
            VpPayload::V1(_) => None,
            VpPayload::V2(_) => Self::CTY,
        }
    }

    fn accepts_typ(typ: Option<&JwtType>) -> bool {
        matches!(typ, None | Some(JwtType::Vp | JwtType::VpLd | JwtType::Jwt))
    }
}
