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
use crate::capabilities::{Signer, Verifier};
use crate::config::types::HostType;
use crate::data::entities::received::grant;
use crate::data::entities::shared::issuance;
use crate::errors::{BadFormat, Errors, MissingAction, Outcome};
use crate::types::gnap::grant_request::GrantRequestKind;
use crate::types::gnap::grant_request::client::{Client, KeyMaterial};
use crate::types::issuing::{
    AuthServerMetadata, CredReqProof, CredentialRequest, DidPossession, GiveVC, IssuedCredential,
    IssuerMetadata, IssuingToken, VcCredOffer, VcTransmissionOffer,
};
use crate::types::jwt::{Jwt, VCJwtClaims};
use crate::types::keys::{PrivateKey, SigningCtx};
use crate::types::secrets::PemHelper;
use crate::types::vcs::{BuildCtx, VcType, VcTypeConfig};
use crate::utils::{expect_from_env, is_active};
use async_trait::async_trait;
use std::{format, vec};
use tracing::info;

#[async_trait]
pub trait IssuerTrait: Send + Sync + 'static {
    fn get_cred_offer_data(&self, model: &issuance::Model) -> VcCredOffer;
    fn build_issuance_plan(
        &self,
        id: &str,
        grant_request_kind: GrantRequestKind,
        client: Client,
        available_vcs: &[VcType],
    ) -> Outcome<issuance::Plan>;
    fn generate_issuing_uri(
        &self,
        offer_type: VcTransmissionOffer,
        path: Option<&str>,
    ) -> Outcome<String>;
    fn get_issuer_metadata(&self, path: Option<&str>, vcs: &[VcType]) -> IssuerMetadata;
    fn get_oauth_server_data(&self, path: Option<&str>) -> AuthServerMetadata;
    fn get_token(&self, model: &issuance::Model) -> IssuingToken;
    async fn validate_cred_req(
        &self,
        issuance: &issuance::Model,
        cred_req: CredentialRequest,
        token: &str,
    ) -> Outcome<(String, VcTypeConfig)>;
    async fn issue_cred(&self, claims: &VCJwtClaims) -> Outcome<String>;
}
