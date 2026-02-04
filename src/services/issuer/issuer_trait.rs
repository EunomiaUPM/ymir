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

use crate::data::entities::{issuing, minions, recv_interaction, vc_request};
use crate::types::issuing::{
    AuthServerMetadata, CredentialRequest, DidPossession, GiveVC, IssuerMetadata, IssuingToken,
    TokenRequest, VCCredOffer, WellKnownJwks
};

#[async_trait]
pub trait IssuerTrait: Send + Sync + 'static {
    fn start_vci(&self, req_model: &vc_request::Model) -> issuing::NewModel;
    fn generate_issuing_uri(&self, id: &str) -> String;
    fn get_cred_offer_data(&self, model: &issuing::Model) -> anyhow::Result<VCCredOffer>;
    fn get_issuer_data(&self, path: Option<&str>) -> IssuerMetadata;
    fn get_oauth_server_data(&self, path: Option<&str>) -> AuthServerMetadata;
    fn get_token(&self, model: &issuing::Model) -> IssuingToken;
    fn validate_token_req(
        &self,
        model: &issuing::Model,
        payload: &TokenRequest
    ) -> anyhow::Result<()>;
    async fn issue_cred(&self, claims: Value, did: Option<String>) -> anyhow::Result<GiveVC>;
    async fn validate_cred_req(
        &self,
        model: &mut issuing::Model,
        cred_req: &CredentialRequest,
        token: &str
    ) -> anyhow::Result<()>;
    fn validate_did_possession(
        &self,
        token: &TokenData<DidPossession>,
        kid: &str
    ) -> anyhow::Result<()>;
    fn end(
        &self,
        req_model: &vc_request::Model,
        int_model: &recv_interaction::Model,
        iss_model: &issuing::Model
    ) -> anyhow::Result<minions::NewModel>;
    async fn get_jwks_data(&self) -> anyhow::Result<WellKnownJwks>;
}
