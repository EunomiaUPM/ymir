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

use std::str::FromStr;
use std::sync::Arc;

use anyhow::bail;
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, EncodingKey, Header, TokenData, encode};
use rsa::RsaPublicKey;
use rsa::pkcs1::DecodeRsaPublicKey;
use serde_json::Value;
use tracing::{error, info};
use urlencoding;

use super::super::IssuerTrait;
use super::config::{BasicIssuerConfig, BasicIssuerConfigTrait};
use crate::data::entities::{issuing, minions, recv_interaction, vc_request};
use crate::errors::{ErrorLogTrait, Errors};
use crate::services::client::ClientTrait;
use crate::services::vault::VaultTrait;
use crate::services::vault::vault_rs::VaultService;
use crate::types::errors::BadFormat;
use crate::types::issuing::{
    AuthServerMetadata, CredentialRequest, DidPossession, GiveVC, IssuerMetadata, IssuingToken,
    TokenRequest, VCCredOffer, WellKnownJwks,
};
use crate::types::secrets::StringHelper;
use crate::types::vcs::VcType;
use crate::utils::{
    expect_from_env, get_from_opt, has_expired, is_active, trim_4_base, validate_token,
};

pub struct BasicIssuerService {
    config: BasicIssuerConfig,
    client: Arc<dyn ClientTrait>,
    vault: Arc<VaultService>,
}

impl BasicIssuerService {
    pub fn new(
        config: BasicIssuerConfig,
        client: Arc<dyn ClientTrait>,
        vault: Arc<VaultService>,
    ) -> BasicIssuerService {
        BasicIssuerService { config, client, vault }
    }
}

#[async_trait]
impl IssuerTrait for BasicIssuerService {
    fn start_vci(&self, model: &vc_request::Model) -> issuing::NewModel {
        info!("Starting OIDC4VCI");
        let host = format!("{}{}/issuer", self.config.get_host(), self.config.get_api_path());
        let aud = match self.config.is_local() {
            true => host.replace("127.0.0.1", "host.docker.internal"),
            false => host,
        };

        issuing::NewModel {
            id: model.id.clone(),
            name: model.participant_slug.clone(),
            vc_type: model.vc_type.clone(),
            aud,
        }
    }

    fn generate_issuing_uri(&self, id: &str) -> String {
        let semi_host = format!(
            "{}{}/issuer",
            self.config.get_host_without_protocol(),
            self.config.get_api_path()
        );
        let host = format!("{}{}/issuer", self.config.get_host(), self.config.get_api_path());
        let (semi_host, host) = match self.config.is_local() {
            true => {
                let a = semi_host.replace("127.0.0.1", "host.docker.internal");
                let b = host.replace("127.0.0.1", "host.docker.internal");
                (a, b)
            }
            false => (semi_host, host),
        };
        let h_host = format!("{}/credentialOffer?id={}", host, &id);
        let encoded_host = urlencoding::encode(h_host.as_str());
        let uri = format!(
            "openid-credential-offer://{}/?credential_offer_uri={}",
            semi_host, encoded_host
        );
        info!("Issuing uri: {}", uri);
        uri
    }

    fn get_cred_offer_data(&self, model: &issuing::Model) -> anyhow::Result<VCCredOffer> {
        info!("Retrieving credential offer data");

        let issuer = format!("{}{}/issuer", self.config.get_host(), self.config.get_api_path());
        let issuer = match self.config.is_local() {
            true => issuer.replace("127.0.0.1", "host.docker.internal"),
            false => issuer,
        };

        let vc_type = VcType::from_str(&model.vc_type)?;

        let offer = match model.step {
            true => VCCredOffer::new(issuer, model.tx_code.clone(), vc_type),
            false => VCCredOffer::new(issuer, model.pre_auth_code.clone(), vc_type),
        };

        Ok(offer)
    }

    fn get_issuer_data(&self) -> IssuerMetadata {
        info!("Retrieving issuer data");
        let host = format!("{}{}/issuer", self.config.get_host(), self.config.get_api_path());
        let host = match self.config.is_local() {
            true => host.replace("127.0.0.1", "host.docker.internal"),
            false => host,
        };
        IssuerMetadata::new(&host)
    }

    fn get_oauth_server_data(&self) -> AuthServerMetadata {
        info!("Retrieving oauth server data");

        let host = format!("{}{}/issuer", self.config.get_host(), self.config.get_api_path());
        let host = match self.config.is_local() {
            true => host.replace("127.0.0.1", "host.docker.internal"),
            false => host,
        };

        AuthServerMetadata::new(&host)
    }

    fn get_token(&self, model: &issuing::Model) -> IssuingToken {
        info!("Giving token");
        IssuingToken::new(model.token.clone())
    }
    fn validate_token_req(
        &self,
        model: &issuing::Model,
        payload: &TokenRequest,
    ) -> anyhow::Result<()> {
        info!("Validating token vc_request");

        if let Some(tx_code) = &payload.tx_code {
            if model.tx_code != *tx_code {
                let error = Errors::forbidden_new("tx_code does not match");
                error!("{}", error.log());
                bail!(error)
            }
        }

        if model.pre_auth_code != payload.pre_authorized_code {
            let error = Errors::forbidden_new("pre_auth_code does not match");
            error!("{}", error.log());
            bail!(error)
        }

        Ok(())
    }

    async fn issue_cred(&self, claims: Value) -> anyhow::Result<GiveVC> {
        info!("Issuing cred");

        let did = self.config.get_did();
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(did.to_string());
        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: StringHelper = self.vault.read(None, &priv_key).await?;

        let key = EncodingKey::from_rsa_pem(priv_key.data().as_bytes()).map_err(|e| {
            let error = Errors::format_new(
                BadFormat::Unknown,
                &format!("Error parsing private key: {}", e.to_string()),
            );
            error!("{}", error.log());
            error
        })?;

        let vc_jwt = encode(&header, &claims, &key).map_err(|e| {
            let error = Errors::format_new(
                BadFormat::Unknown,
                &format!("Error signing token: {}", e.to_string()),
            );
            error!("{}", error.log());
            error
        })?;

        Ok(GiveVC { format: "jwt_vc_json".to_string(), credential: vc_jwt })
    }

    async fn validate_cred_req(
        &self,
        model: &mut issuing::Model,
        cred_req: &CredentialRequest,
        token: &str,
    ) -> anyhow::Result<()> {
        info!("Validating credential vc_request");

        if model.token != token {
            let error = Errors::forbidden_new("tx_code does not match");
            error!("{}", error.log());
            bail!(error)
        }

        if cred_req.format != "jwt_vc_json" {
            let error = Errors::format_new(
                BadFormat::Received,
                &format!("Cannot issue a credentia with format: {}", cred_req.format),
            );
            error!("{}", error.log());
            bail!(error)
        }

        if cred_req.proof.proof_type != "jwt" {
            let error = Errors::format_new(
                BadFormat::Received,
                &format!("Cannot validate proof with type: {}", cred_req.proof.proof_type),
            );
            error!("{}", error.log());
            bail!(error)
        }

        let did = self.config.get_did();
        let (token, kid) = validate_token::<DidPossession>(
            &cred_req.proof.jwt,
            Some(&model.aud),
            self.client.clone(),
        )
        .await?;
        self.validate_did_possession(&token, &kid)?;
        model.holder_did = Some(kid);
        model.issuer_did = Some(did);
        is_active(token.claims.iat)?;
        has_expired(token.claims.exp)?;
        Ok(())
    }

    fn validate_did_possession(
        &self,
        token: &TokenData<DidPossession>,
        kid: &str,
    ) -> anyhow::Result<()> {
        info!("Validating did possession");
        if token.claims.iss != token.claims.sub && token.claims.sub != kid {
            let error = Errors::forbidden_new("Invalid proof of did possession");
            error!("{}", error.log());
            bail!(error)
        }
        Ok(())
    }
    fn end(
        &self,
        req_model: &vc_request::Model,
        int_model: &recv_interaction::Model,
        iss_model: &issuing::Model,
    ) -> anyhow::Result<minions::NewModel> {
        let did = get_from_opt(&iss_model.holder_did, "did")?;
        let base_url = trim_4_base(&int_model.uri);
        Ok(minions::NewModel {
            participant_id: did,
            participant_slug: req_model.participant_slug.clone(),
            vc_uri: req_model.vc_uri.clone(),
            participant_type: "Minion".to_string(),
            base_url: Some(base_url),
            is_vc_issued: false,
            is_me: false,
        })
    }

    async fn get_jwks_data(&self) -> anyhow::Result<WellKnownJwks> {
        info!("Retrieving jwks data");

        let pub_key = expect_from_env("VAULT_APP_PUB_PKEY");
        let pub_key: StringHelper = self.vault.read(None, &pub_key).await?;

        let key = RsaPublicKey::from_pkcs1_pem(&pub_key.data()).map_err(|e| {
            let error = Errors::format_new(
                BadFormat::Unknown,
                &format!("Error parsing public key: {}", e.to_string()),
            );
            error!("{}", error.log());
            error
        })?;
        Ok(WellKnownJwks::new(key))
    }
}
