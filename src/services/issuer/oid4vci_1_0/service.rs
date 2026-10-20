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
use tokio::sync::RwLock;

use async_trait::async_trait;
use tracing::info;
use urlencoding;

use super::super::IssuerTrait;
use super::IssuerConfig;
use crate::capabilities::{Signer, Verifier};
use crate::config::traits::HostsConfigTrait;
use crate::config::types::HostType;
use crate::data::entities::shared::issuance;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::services::vault::{VaultService, VaultTrait};
use crate::types::dids::Did;
use crate::types::gnap::grant_request::GrantRequestKind;
use crate::types::gnap::grant_request::client::{Client, KeyMaterial};
use crate::types::issuance::{
    AuthServerMetadata, CredReqProof, CredentialRequest, DidPossession, IssuerMetadata,
    IssuingToken, VcCredOffer, VcTransmissionOffer,
};
use crate::types::jwt::{Jwt, JwtClaims, JwtHeaderTrait};
use crate::types::keys::{PrivateKey, SigningCtx};
use crate::types::secrets::PemHelper;
use crate::types::oauth::RolePath;
use crate::types::vcs::{BuildCtx, VcPayload, VcType, VcTypeConfig};
use crate::types::wallet::Identity;
use crate::utils::{is_active, Verified};

/// Core Implementation of the OpenID4VCI (v1.0) Credential Issuer Service.
///
/// Implements the OpenID for Verifiable Credential Issuance v1.0 specification.
/// Backed by a configured server environment, an active decentralized Identity reference (DID/Keys),
/// and an abstraction over an unsealed secret storage Vault.
pub struct IssuerService {
    config: IssuerConfig,
    identity: Arc<RwLock<Identity>>,
    vault: Arc<VaultService>,
}

impl IssuerService {
    pub fn new(
        config: IssuerConfig,
        vault: Arc<VaultService>,
        identity: Arc<RwLock<Identity>>,
    ) -> Self {
        Self {
            config,
            vault,
            identity,
        }
    }
}

#[async_trait]
impl IssuerTrait for IssuerService {
    async fn build_issuance_plan(
        &self,
        role: &RolePath,
        id: &str,
        grant_request_kind: GrantRequestKind,
        client: Client,
        available_vcs: &[VcType],
    ) -> Outcome<issuance::Plan> {
        let vc_req = match grant_request_kind {
            GrantRequestKind::AccessToken { .. } => {
                return Err(Errors::format(
                    BadFormat::Received,
                    "Unable to issue tokens, just credentials",
                    None,
                ));
            }
            GrantRequestKind::CredentialRequest { credential_request } => credential_request,
        };

        let participant_nick = client.class_id.as_deref().ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "Missing field class_id in the petition",
                None,
            )
        })?;

        let vc_configs: Vec<VcTypeConfig> = vc_req
            .credential_configurations
            .into_iter()
            .filter(|vc| vc.is_supported())
            .filter(|vc| available_vcs.contains(vc.vc_type()))
            .collect();

        let cert = match client.key.material {
            KeyMaterial::Jwk { .. } => None,
            KeyMaterial::Cert { cert } => Some(cert),
        };
        let aud = self.config.get_host(HostType::Http);

        let build_ctx = BuildCtx::base(participant_nick, cert);

        let lock = self.identity.read().await;
        let issuer_did = lock.did().id().to_string();

        let issuance = issuance::Plan {
            role: role.clone(),
            id: id.to_string(),
            subject_name: participant_nick.to_string(),
            vc_type_config: vc_configs,
            build_ctx,
            aud,
            issuer_did,
        };

        Ok(issuance)
    }

    fn get_cred_offer_data(&self, model: &issuance::Model) -> VcCredOffer {
        info!("Retrieving credential offer data");

        VcCredOffer::pre_authorized(
            self.config.get_host(HostType::Http),
            &model.pre_auth_code,
            &model.vc_type_config,
        )
    }

    fn generate_issuing_uri(&self, offer_type: VcTransmissionOffer) -> Outcome<String> {
        let api_path = self.config.get_api_path();
        let host = format!(
            "{}{}/issuer",
            self.config.get_host(HostType::Http),
            api_path,
        );

        match offer_type {
            VcTransmissionOffer::ByReference(id) => {
                let credential_offer_endpoint = format!("{}/credentialOffer?id={}", host, id);
                let encoded = urlencoding::encode(&credential_offer_endpoint);

                let uri = format!(
                    "openid-credential-offer://?credential_offer_uri={}",
                    encoded
                );
                info!("Issuing uri (by reference): {uri}");
                Ok(uri)
            }
            VcTransmissionOffer::ByValue(cred_offer) => {
                let json_string = serde_json::to_string(&cred_offer)?;

                let encoded_json = urlencoding::encode(&json_string);

                let uri = format!(
                    "openid-credential-offer://?credential_offer={}",
                    encoded_json
                );
                info!("Issuing uri (embedded/by value): {uri}");
                Ok(uri)
            }
        }
    }

    fn get_issuer_metadata(&self, vcs: &[VcType]) -> IssuerMetadata {
        let (host, api_path) = self.metadata_hosts();
        IssuerMetadata::new(&host, &api_path, vcs)
    }

    fn get_oauth_server_data(&self) -> AuthServerMetadata {
        let (host, api_path) = self.metadata_hosts();
        AuthServerMetadata::new(&host, &api_path)
    }

    fn get_token(&self, model: &issuance::Model) -> IssuingToken {
        info!("Giving token");
        IssuingToken::new(
            &model.token,
            Some(model.nonce.clone()),
            model.token_expiration as u32,
        )
    }
    async fn validate_cred_req(
        &self,
        issuance: &issuance::Model,
        cred_req: CredentialRequest,
        token: &str,
    ) -> Outcome<(Did, VcTypeConfig)> {
        info!("Validating credential request");

        if issuance.token != token {
            return Err(Errors::forbidden("token does not match", None));
        }
        let vc_config = requested_config(issuance, &cred_req)?;

        let proof = proof_jwt(cred_req)?;
        let proof = Verifier::verify_enveloped(proof, Some(issuance.aud.clone())).await?;
        validate_did_possession(&proof, issuance)?;

        Ok((proof.kid()?.did().clone(), vc_config))
    }

    async fn sign_claims(&self, claims: &JwtClaims<VcPayload>) -> Outcome<Jwt<VcPayload>> {
        info!("Issuing credential");

        let (did, key_ref) = {
            let identity = self.identity.read().await;
            (identity.did().clone(), identity.key_ref().clone())
        };
        let pem_helper: PemHelper = self.vault.read(None, key_ref.internal()).await?;
        let key = PrivateKey::try_from(pem_helper)?;
        let sig_ctx = SigningCtx::new(did, key, key_ref.fragment().to_string());

        Signer::sign_enveloped(&sig_ctx, claims)
    }
}

// ===== Internal helpers ======================================================

impl IssuerService {
    fn metadata_hosts(&self) -> (String, String) {
        let host = self.config.get_host(HostType::Http);
        let api_path = format!("{}/issuer", self.config.get_api_path());
        (host, api_path)
    }
}

// ===== Free helpers ==========================================================

/// The configuration asked for, which must be one of those offered.
fn requested_config(issuance: &issuance::Model, cred_req: &CredentialRequest) -> Outcome<VcTypeConfig> {
    let vc_config = cred_req.credential_configuration_id.clone().ok_or_else(|| {
        Errors::format(BadFormat::Received, "credential configuration id is missing", None)
    })?;
    if !issuance.vc_type_config.contains(&vc_config) {
        return Err(Errors::format(BadFormat::Received, "Credential config does not match", None));
    }
    Ok(vc_config)
}

/// The proof of possession, only as a JWT.
fn proof_jwt(cred_req: CredentialRequest) -> Outcome<Jwt<DidPossession>> {
    match cred_req.proof {
        Some(CredReqProof::Jwt { jwt }) => Jwt::<DidPossession>::parse(&jwt),
        Some(_) => Err(Errors::format(
            BadFormat::Received,
            "Proof method does not match with requested one",
            None,
        )),
        None => Err(Errors::format(BadFormat::Received, "Proof missing in request", None)),
    }
}

/// OpenID4VCI 1.0 F.1: the `c_nonce` echoed, `iat` present and not in the future, and `iss`, if
/// any, the wallet that signed.
fn validate_did_possession(
    proof: &Verified<Jwt<DidPossession>>,
    issuance: &issuance::Model,
) -> Outcome<()> {
    info!("Validating proof of possession");

    match proof.claims().nonce() {
        Some(nonce) if nonce == issuance.nonce => {}
        Some(_) => return Err(Errors::security("Proof nonce does not match the c_nonce", None)),
        None => return Err(Errors::security("Proof lacks the c_nonce", None)),
    }

    let iat = proof
        .claims()
        .iat()
        .ok_or_else(|| Errors::format(BadFormat::Received, "Proof lacks iat", None))?;
    is_active(iat)?;

    if let Some(iss) = proof.claims().iss() {
        if iss != proof.kid()?.did().id() {
            return Err(Errors::security("Proof iss is not the key's DID", None));
        }
    }
    Ok(())
}
