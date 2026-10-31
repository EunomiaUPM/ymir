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

use crate::types::crypto::{HasProofPurpose, ProofPurpose};
use crate::types::jwt::{HasJwtType, JwtType};
use serde::{Deserialize, Serialize};

// ════════════════════════════════════════════════════════════════════════════════
//   DidPossession
// ════════════════════════════════════════════════════════════════════════════════

/// OpenID4VCI 1.0 proof of possession (F.1): every claim it uses (`aud`, `iat`, `nonce`, `iss`)
/// is a registered one, in [`JwtClaims`](crate::types::jwt::JwtClaims); the type is what says it
/// is a proof (its `typ` and the key's purpose).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DidPossession {}

impl HasProofPurpose for DidPossession {
    const PURPOSE: ProofPurpose = ProofPurpose::Authentication;
}

/// Required as is by OpenID4VCI 1.0 (F.1).
impl HasJwtType for DidPossession {
    const TYP: JwtType = JwtType::Oid4vciProof;
}
