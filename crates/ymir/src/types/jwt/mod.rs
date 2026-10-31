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

// mod vp;

mod header;
mod jwt;
mod jwt_type;
// mod vc_claims;
// mod vc_claims_builder;
// mod w3c_vc;
// mod vp_claims_builder;
// mod claims_base;
mod claims;
mod key;

pub use header::*;
pub use jwt::*;
pub use jwt_type::{HasJwtType, JwtType};
// pub use vc_claims::*;
// pub use vc_claims_builder::*;
// pub use vp::{VPJwtClaims, VpJwtClaimsV1, VpJwtClaimsV2};
// pub use w3c_vc::*;
// pub use vp_claims_builder::VpJwtClaimsBuilder;
// VC and VP claims live in `types::vcs::jwt` and `types::vps::jwt`.
// pub use claims_base::{JwtClaimsBase, JwtBaseClaimsTrait};
pub use claims::JwtClaims;
pub use key::JwtKey;