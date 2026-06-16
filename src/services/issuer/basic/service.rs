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

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tracing::info;
use urlencoding;

use super::super::IssuerTrait;
use super::BasicIssuerConfig;
use crate::capabilities::{Kid, Signer, Verifier};
use crate::config::traits::HostsConfigTrait;
use crate::config::types::HostType;
use crate::data::entities::{issuing, minions, recv_interaction, vc_request};
use crate::errors::{BadFormat, Errors, MissingAction, Outcome};
use crate::services::vault::{VaultService, VaultTrait};
use crate::types::issuing::{
    AuthServerMetadata, CredentialRequest, DidPossession, GiveVC, IssuerMetadata, IssuingToken,
    TokenRequest, VcCredOffer,
};
use crate::types::jwt::Jwt;
use crate::types::keys::{PrivateKey, PublicKey, SigningCtx};
use crate::types::secrets::PemHelper;
use crate::types::vcs::VcType;
use crate::types::wallet::Identity;
use crate::utils::{expect_from_env, has_expired, is_active, trim_4_base};

pub struct BasicIssuerService {
    config: BasicIssuerConfig,
    identity: Arc<Identity>,
    vault: Arc<VaultService>,
}

impl BasicIssuerService {
    pub fn new(
        config: BasicIssuerConfig,
        vault: Arc<VaultService>,
        identity: Arc<Identity>,
    ) -> Self {
        Self {
            config,
            vault,
            identity,
        }
    }
}

#[async_trait]
impl IssuerTrait for BasicIssuerService {
    fn start_vci(&self, model: &vc_request::Model) -> issuing::NewModel {
        info!("Starting OIDC4VCI");
        let aud = self.config.get_host(HostType::Http);
        let uri = self.generate_issuing_uri(&model.id, None);
        issuing::NewModel {
            id: model.id.clone(),
            name: model.participant_slug.clone(),
            vc_type: model.vc_type.clone(),
            aud,
            uri,
        }
    }

    fn generate_issuing_uri(&self, id: &str, path: Option<&str>) -> String {
        let path = path.unwrap_or("issuer");
        let api_path = self.config.get_api_path();
        let semi_host = format!(
            "{}{}/{}",
            self.config.get_host_without_protocol(HostType::Http),
            api_path,
            path
        );
        let host = format!(
            "{}{}/{}",
            self.config.get_host(HostType::Http),
            api_path,
            path
        );

        let credential_offer_endpoint = format!("{}/credentialOffer?id={}", host, id);
        let encoded = urlencoding::encode(credential_offer_endpoint.as_str());
        let uri = format!(
            "openid-credential-offer://{}/?credential_offer_uri={}",
            semi_host, encoded
        );
        info!("Issuing uri: {uri}");
        uri
    }

    fn get_cred_offer_data(&self, model: &issuing::Model) -> Outcome<VcCredOffer> {
        info!("Retrieving credential offer data");
        VcCredOffer::new(
            self.config.get_host(HostType::Http),
            &model.pre_auth_code,
            &model.vc_type,
        )
    }

    fn get_issuer_data(&self, path: Option<&str>, vcs: Option<&[VcType]>) -> IssuerMetadata {
        info!("Retrieving issuer data");
        let (base_host, host_path) = self.metadata_hosts(path);
        IssuerMetadata::new(&base_host, &host_path, vcs)
    }

    fn get_oauth_server_data(
        &self,
        path: Option<&str>,
        vcs: Option<&[VcType]>,
    ) -> AuthServerMetadata {
        info!("Retrieving oauth server data");
        let (base_host, host_path) = self.metadata_hosts(path);
        AuthServerMetadata::new(&base_host, &host_path, vcs)
    }

    fn get_token(&self, model: &issuing::Model) -> IssuingToken {
        info!("Giving token");
        IssuingToken::new(model.token.clone())
    }

    fn validate_token_req(&self, model: &issuing::Model, payload: &TokenRequest) -> Outcome<()> {
        info!("Validating token request");

        // if let Some(tx_code) = &payload.tx_code {
        //     if model.tx_code != *tx_code {
        //         return Err(Errors::forbidden("tx_code does not match", None));
        //     }
        // }

        if model.pre_auth_code != payload.pre_authorized_code {
            return Err(Errors::forbidden("pre_auth_code does not match", None));
        }
        Ok(())
    }

    async fn validate_cred_req(
        &self,
        model: &mut issuing::Model,
        cred_req: &CredentialRequest,
        token: &str,
    ) -> Outcome<()> {
        info!("Validating credential request");

        if model.token != token {
            return Err(Errors::forbidden("token does not match", None));
        }
        if cred_req.format != "jwt_vc_json" {
            return Err(Errors::format(
                BadFormat::Received,
                format!("Cannot issue a credential with format: {}", cred_req.format),
                None,
            ));
        }
        if cred_req.proof.proof_type != "jwt" {
            return Err(Errors::format(
                BadFormat::Received,
                format!(
                    "Cannot validate proof with type: {}",
                    cred_req.proof.proof_type
                ),
                None,
            ));
        }

        let proof_jwt = Jwt::parse(&cred_req.proof.jwt)?;

        let (kid, claims) =
            Verifier::verify_enveloped::<DidPossession>(&proof_jwt, Some(&model.aud)).await?;

        validate_did_possession(&claims, &kid)?;
        is_active(claims.iat)?;
        has_expired(claims.exp)?;

        model.holder_did = Some(kid.did().id().to_string());
        model.issuer_did = Some(self.identity.did().id().to_string());
        Ok(())
    }

    async fn issue_cred(&self, claims: &Value) -> Outcome<GiveVC> {
        info!("Issuing credential");
        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let pem_helper: PemHelper = self.vault.read(None, &priv_key).await?;
        let key = PrivateKey::try_from(pem_helper)?;
        let did = self.identity.did().clone();
        let keys_id = self.identity.keys_id().first().cloned().ok_or_else(|| {
            Errors::missing_action(MissingAction::Key, "No key within did Document", None)
        })?;

        let sig_ctx = SigningCtx::new(did, key, keys_id.fragment().to_string());

        let vc_jwt = Signer::sign_enveloped(&sig_ctx, "vc+ld+json+jwt", "vc+ld+json", claims)?;
        Ok(GiveVC {
            format: "jwt_vc_json".to_string(),
            credential: vc_jwt.to_string(),
        })
    }

    fn end(
        &self,
        req_model: &vc_request::Model,
        int_model: &recv_interaction::Model,
        iss_model: &issuing::Model,
    ) -> Outcome<minions::NewModel> {
        let did = iss_model
            .holder_did
            .as_ref()
            .ok_or_else(|| Errors::format(BadFormat::Received, "Missing holder_did", None))?;
        let base_url = trim_4_base(&int_model.uri);
        Ok(minions::NewModel {
            participant_id: did.clone(),
            participant_slug: req_model.participant_slug.clone(),
            vc_uri: req_model.vc_uri.clone(),
            participant_type: "Agent".to_string(),
            base_url: Some(base_url),
            is_vc_issued: true,
            is_me: false,
        })
    }

    async fn get_jwks_data(&self) -> Outcome<Value> {
        info!("Retrieving jwks data");
        let pub_key = expect_from_env("VAULT_APP_PUB_PKEY");
        let pub_key: PemHelper = self.vault.read(None, &pub_key).await?;
        let key = PublicKey::try_from(pub_key)?;
        Ok(key.public_jwk())
    }
}

// ===== Internal helpers ======================================================

impl BasicIssuerService {
    fn metadata_hosts(&self, path: Option<&str>) -> (String, String) {
        let path = path.unwrap_or("/issuer");
        let full_path = format!("{}{}", self.config.get_api_path(), path);
        let base_host = self.config.get_host(HostType::Http);
        let host_path = format!("{}{}", self.config.get_host(HostType::Http), full_path);
        (base_host, host_path)
    }
}

// ===== Free helpers ==========================================================

fn validate_did_possession(claims: &DidPossession, kid: &Kid) -> Outcome<()> {
    info!("Validating did possession");
    if claims.iss != kid.did().id() || claims.sub != kid.did().id() {
        return Err(Errors::forbidden("Invalid proof of did possession", None));
    }
    Ok(())
}
