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

use crate::data::entities::{issuing, minions, recv_interaction, vc_request};
use crate::errors::Outcome;
use crate::types::issuing::{
    AuthServerMetadata, CredentialRequest, GiveVC, IssuerMetadata, IssuingToken, TokenRequest,
    VcCredOffer,
};
use crate::types::vcs::VcType;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait IssuerTrait: Send + Sync + 'static {
    fn start_vci(&self, req_model: &vc_request::Model) -> issuing::NewModel;
    fn generate_issuing_uri(&self, id: &str, path: Option<&str>) -> String;
    fn get_cred_offer_data(&self, model: &issuing::Model) -> Outcome<VcCredOffer>;
    fn get_issuer_data(&self, path: Option<&str>, vcs: Option<&[VcType]>) -> IssuerMetadata;
    fn get_oauth_server_data(
        &self,
        path: Option<&str>,
        vcs: Option<&[VcType]>,
    ) -> AuthServerMetadata;
    fn get_token(&self, model: &issuing::Model) -> IssuingToken;
    fn validate_token_req(&self, model: &issuing::Model, payload: &TokenRequest) -> Outcome<()>;
    async fn validate_cred_req(
        &self,
        model: &mut issuing::Model,
        cred_req: &CredentialRequest,
        token: &str,
    ) -> Outcome<()>;
    async fn issue_cred(&self, claims: &Value) -> Outcome<GiveVC>;
    fn end(
        &self,
        req_model: &vc_request::Model,
        int_model: &recv_interaction::Model,
        iss_model: &issuing::Model,
    ) -> Outcome<minions::NewModel>;
    async fn get_jwks_data(&self) -> Outcome<Value>;
}
