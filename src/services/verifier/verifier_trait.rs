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

use async_trait::async_trait;
use crate::data::entities::received::verification::{Model, Plan};
use crate::errors::Outcome;
use crate::types::vcs::VPDef;

#[async_trait]
pub trait VerifierTrait: Send + Sync + 'static {
    fn build_vp_plan(&self, id: &str) -> Outcome<Plan>;
    fn generate_verification_uri(&self, model: &Model) -> String;
    fn generate_vpd(&self, ver_model: &Model) -> Outcome<VPDef>;
    async fn verify_all(&self, ver_model: &mut Model, vp_token: &str) -> Outcome<()>;
}
