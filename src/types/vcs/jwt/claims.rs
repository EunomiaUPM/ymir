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
use crate::types::vcs::doc::VcDocument;

/// What a VC's JWT carries besides the registered claims, per data model version.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum VcPayload {
    V1(VcPayloadV1),
    V2(VcDocument),
}

/// VC DM 1.1 §6.3.1: the credential goes in the `vc` claim.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VcPayloadV1 {
    pub vc: VcDocument,
}

impl VcPayload {
    pub fn version(&self) -> W3cDataModelVersion {
        match self {
            VcPayload::V1(_) => W3cDataModelVersion::V1,
            VcPayload::V2(_) => W3cDataModelVersion::V2,
        }
    }
    pub fn vc_doc(&self) -> &VcDocument {
        match self {
            VcPayload::V1(v1) => &v1.vc,
            VcPayload::V2(doc) => doc,
        }
    }
}

impl JwtClaims<VcPayload> {
    pub fn vc_doc(&self) -> &VcDocument {
        self.body().vc_doc()
    }
}

/// The issuer's key must be authorized for assertions, not for authentication.
impl HasProofPurpose for VcPayload {
    const PURPOSE: ProofPurpose = ProofPurpose::AssertionMethod;
}

/// VC DM 1.1 §6.3.1: `JWT`; VC-JOSE-COSE §3.1.1 (VC DM 2.0): `vc+jwt` + `cty: vc`. Also reads
/// Gaia-X 24.07's `vc+ld+json+jwt`.
impl HasJwtType for VcPayload {
    const TYP: JwtType = JwtType::Vc;
    const CTY: Option<&'static str> = Some("vc");

    fn jwt_type(&self) -> JwtType {
        match self {
            VcPayload::V1(_) => JwtType::Jwt,
            VcPayload::V2(_) => JwtType::Vc,
        }
    }

    fn content_type(&self) -> Option<&'static str> {
        match self {
            VcPayload::V1(_) => None,
            VcPayload::V2(_) => Self::CTY,
        }
    }

    fn accepts_typ(typ: Option<&JwtType>) -> bool {
        matches!(typ, None | Some(JwtType::Vc | JwtType::VcLd | JwtType::Jwt))
    }
}
