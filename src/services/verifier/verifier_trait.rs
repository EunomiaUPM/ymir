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

use async_trait::async_trait;
use jsonwebtoken::TokenData;
use serde_json::Value;

use crate::data::entities::recv_verification::{Model, NewModel};
use crate::types::vcs::VPDef;

#[async_trait]
pub trait VerifierTrait: Send + Sync + 'static {
    fn start_vp(&self, id: &str) -> anyhow::Result<NewModel>;
    fn generate_verification_uri(&self, model: Model) -> String;
    fn generate_vpd(&self, ver_model: Model) -> VPDef;
    async fn verify_all(&self, ver_model: &mut Model, vp_token: String) -> anyhow::Result<()>;
    async fn verify_vp(
        &self,
        model: &mut Model,
        vp_token: &str
    ) -> anyhow::Result<(Vec<String>, String)>;
    async fn verify_vc(&self, vc_token: &str, holder: &str) -> anyhow::Result<()>;
    async fn validate_token(
        &self,
        vp_token: &str,
        audience: Option<&str>
    ) -> anyhow::Result<(TokenData<Value>, String)>;
    fn validate_nonce(&self, model: &Model, token: &TokenData<Value>) -> anyhow::Result<()>;
    fn validate_vp_subject(
        &self,
        model: &mut Model,
        token: &TokenData<Value>,
        kid: &str
    ) -> anyhow::Result<()>;
    fn validate_vc_sub(&self, token: &TokenData<Value>, holder: &str) -> anyhow::Result<()>;
    fn validate_vp_id(&self, model: &Model, token: &TokenData<Value>) -> anyhow::Result<()>;
    fn validate_holder(&self, model: &Model, token: &TokenData<Value>) -> anyhow::Result<()>;
    fn validate_issuer(&self, token: &TokenData<Value>, kid: &str) -> anyhow::Result<()>;
    fn validate_vc_id(&self, token: &TokenData<Value>) -> anyhow::Result<()>;
    fn validate_valid_from(&self, token: &TokenData<Value>) -> anyhow::Result<()>;
    fn validate_valid_until(&self, token: &TokenData<Value>) -> anyhow::Result<()>;
    fn retrieve_vcs(&self, token: TokenData<Value>) -> anyhow::Result<Vec<String>>;
}
