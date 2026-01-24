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

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::bail;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jsonwebtoken::{TokenData, Validation};
use serde_json::Value;
use tracing::{error, info};
use urlencoding::encode;

use super::super::VerifierTrait;
use super::config::{BasicVerifierConfig, BasicVerifierConfigTrait};
use crate::capabilities::DidResolver;
use crate::data::entities::recv_verification::{Model, NewModel};
use crate::errors::{ErrorLogTrait, Errors};
use crate::services::client::ClientTrait;
use crate::types::errors::BadFormat;
use crate::types::vcs::VPDef;
use crate::utils::{get_claim, get_opt_claim};

pub struct BasicVerifierService {
    client: Arc<dyn ClientTrait>,
    config: BasicVerifierConfig,
}

impl BasicVerifierService {
    pub fn new(client: Arc<dyn ClientTrait>, config: BasicVerifierConfig) -> BasicVerifierService {
        BasicVerifierService { client, config }
    }
}

#[async_trait]
impl VerifierTrait for BasicVerifierService {
    fn start_vp(&self, id: &str) -> anyhow::Result<NewModel> {
        info!("Managing OIDC4VP");
        let host_url = self.config.get_host();
        let host_url = match self.config.is_local() {
            true => host_url.replace("127.0.0.1", "host.docker.internal"),
            false => host_url,
        };

        let client_id = format!("{}/verify", &host_url);
        let requested_vcs = self.config.get_requested_vcs();
        if requested_vcs.is_empty() {
            let error = Errors::unauthorized_new("Unable to verify following oidc4vp");
            error!("{}", error.log());
            bail!(error)
        }
        let vc_type = serde_json::to_string(&requested_vcs)?;
        let new_verification_model = NewModel { id: id.to_string(), audience: client_id, vc_type };

        Ok(new_verification_model)
    }

    fn generate_verification_uri(&self, model: Model) -> String {
        info!("Generating verification exchange URI");

        let host_url = self.config.get_host();
        let host_url = format!("{}{}/verifier", host_url, self.config.get_api_path());
        let host_url = match self.config.is_local() {
            true => host_url.replace("127.0.0.1", "host.docker.internal"),
            false => host_url,
        };

        let base_url = "openid4vp://authorize";
        let encoded_client_id = encode(&model.audience);
        let presentation_definition_uri = format!("{}/pd/{}", &host_url, model.state);
        let encoded_presentation_definition_uri = encode(&presentation_definition_uri);
        let response_uri = format!("{}/verify/{}", &host_url, model.state);
        let encoded_response_uri = encode(&response_uri);
        let response_type = "vp_token";
        let response_mode = "direct_post";
        let client_id_scheme = "redirect_uri";

        // TODO let client_metadata =
        // r#"{"authorization_encrypted_response_alg":"ECDH-ES","
        // authorization_encrypted_response_enc":"A256GCM"}"#;

        let uri = format!(
            "{}?response_type={}&client_id={}&response_mode={}&presentation_definition_uri={}&client_id_scheme={}&nonce={}&response_uri={}",
            base_url,
            response_type,
            encoded_client_id,
            response_mode,
            encoded_presentation_definition_uri,
            client_id_scheme,
            model.nonce,
            encoded_response_uri
        );
        info!("Uri generated successfully: {}", uri);

        uri
    }

    fn generate_vpd(&self, ver_model: Model) -> VPDef {
        info!("Generating an vp definition");
        VPDef::new(ver_model.id, ver_model.vc_type)
    }

    async fn verify_all(&self, ver_model: &mut Model, vp_token: String) -> anyhow::Result<()> {
        info!("Verifying all");

        let (vcs, holder) = self.verify_vp(ver_model, &vp_token).await?;
        for vc in vcs {
            self.verify_vc(&vc, &holder).await?;
        }
        info!("VP & VC Validated successfully");

        Ok(())
    }

    async fn verify_vp(
        &self,
        model: &mut Model,
        vp_token: &str,
    ) -> anyhow::Result<(Vec<String>, String)> {
        info!("Verifying vp");

        model.vpt = Some(vp_token.to_string());
        let (token, kid) = self.validate_token(vp_token, Some(&model.state)).await?;
        self.validate_nonce(model, &token)?;
        self.validate_vp_subject(model, &token, &kid)?;
        self.validate_vp_id(model, &token)?;
        self.validate_holder(model, &token)?;
        // let id = match token.claims["jti"].as_str() {
        //     Some(data) => data,
        //     None => {
        //         let error = CommonErrors::format_new(
        //             BadFormat::Received,
        //             Some("VPT does not contain the 'jti' field".to_string()),
        //         );
        //         error!("{}", error.log());
        //         bail!(error);
        //     }
        // };

        info!("VP Verification successful");
        let vcs = self.retrieve_vcs(token)?;

        Ok((vcs, kid))
    }

    async fn verify_vc(&self, vc_token: &str, holder: &str) -> anyhow::Result<()> {
        info!("Verifying vc");

        let (token, kid) = self.validate_token(vc_token, None).await?;
        self.validate_issuer(&token, &kid)?;
        self.validate_vc_id(&token)?;
        self.validate_vc_sub(&token, holder)?;

        // if issuers_list.contains(kid) {
        //     // TODO
        //     error!("VCT issuer is not on the trusted issuers list");
        //     bail!("VCT issuer is not on the trusted issuers list");
        // }
        // info!("VCT issuer is on the trusted issuers list");

        self.validate_valid_from(&token)?;
        self.validate_valid_until(&token)?;

        info!("VC Verification successful");

        Ok(())
    }

    async fn validate_token(
        &self,
        vp_token: &str,
        audience: Option<&str>,
    ) -> anyhow::Result<(TokenData<Value>, String)> {
        info!("Validating token");
        let header = jsonwebtoken::decode_header(&vp_token)?;
        let did = header.kid.as_ref().ok_or_else(|| {
            let error = Errors::format_new(BadFormat::Received, "Jwt does not contain a token");
            error!("{}", error.log());
            error
        })?;

        let key = DidResolver::get_key(did, self.client.clone()).await?;
        let (base_did, _) = DidResolver::split_did_id(did);
        let alg = header.alg;

        let mut val = Validation::new(alg);

        val.required_spec_claims = HashSet::new();
        val.validate_exp = false;
        val.validate_nbf = true;

        match audience {
            Some(data) => {
                let audience = format!(
                    "{}{}/verifier/verify/{}",
                    self.config.get_host(),
                    self.config.get_api_path(),
                    data
                );
                let audience = match self.config.is_local() {
                    true => audience.replace("127.0.0.1", "host.docker.internal"),
                    false => audience,
                };
                val.validate_aud = true;
                val.set_audience(&[&(audience)]);
            }
            None => {
                val.validate_aud = false;
            }
        };

        let token = jsonwebtoken::decode::<Value>(&vp_token, &key, &val).map_err(|e| {
            let error =
                Errors::security_new(&format!("VPT signature is incorrect -> {}", e.to_string()));
            error!("{}", error.log());
            error
        })?;

        info!("Token signature is correct");
        Ok((token, base_did.to_string()))
    }

    fn validate_nonce(&self, model: &Model, token: &TokenData<Value>) -> anyhow::Result<()> {
        info!("Validating nonce");

        let nonce = get_claim(&token.claims, vec!["nonce"])?;

        if model.nonce != nonce {
            let error = Errors::security_new("Invalid nonce, it does not match");
            error!("{}", error.log());
            bail!(error);
        }
        info!("VPT Nonce matches");
        Ok(())
    }

    fn validate_vp_subject(
        &self,
        model: &mut Model,
        token: &TokenData<Value>,
        kid: &str,
    ) -> anyhow::Result<()> {
        info!("Validating subject");

        let sub = get_opt_claim(&token.claims, vec!["sub"])?;
        let iss = get_opt_claim(&token.claims, vec!["iss"])?;

        if let Some(sub) = sub {
            if sub != kid {
                let error = Errors::security_new("VPT token subject & kid does not match");
                error!("{}", error.log());
                bail!(error);
            }
            info!("VPT subject & kid matches");
        };

        check_iss(iss, kid)?;

        model.holder = Some(kid.to_string());
        Ok(())
    }

    fn validate_vc_sub(&self, token: &TokenData<Value>, holder: &str) -> anyhow::Result<()> {
        info!("Validating VC subject");

        let sub = get_opt_claim(&token.claims, vec!["sub"])?;
        let cred_sub_id = get_claim(&token.claims, vec!["vc", "CredentialSubject", "id"])?;

        if let Some(sub) = sub {
            if sub != holder {
                let error = Errors::security_new(
                    "VCT token sub, credential subject & VP Holder do not match",
                );
                error!("{}", error.log());
                bail!(error);
            }
            info!("Sub & Holder match");
        };

        if holder != cred_sub_id {
            let error =
                Errors::security_new("VCT token sub, credential subject & VP Holder do not match");
            error!("{}", error.log());
            bail!(error);
        }
        info!("Vc Holder & Holder match");
        Ok(())
    }

    fn validate_vp_id(&self, model: &Model, token: &TokenData<Value>) -> anyhow::Result<()> {
        info!("Validating vp id");

        let vp_id = get_claim(&token.claims, vec!["vp", "id"])?;

        if model.id != vp_id {
            // VALIDATE ID MATCHES JTI
            let error = Errors::security_new("Invalid id, it does not match");
            error!("{}", error.log());
            bail!(error);
        }
        info!("Exchange is valid");
        Ok(())
    }

    fn validate_holder(&self, model: &Model, token: &TokenData<Value>) -> anyhow::Result<()> {
        info!("Validating holder");

        let vp_holder = get_claim(&token.claims, vec!["vp", "holder"])?;

        if model.holder.clone().unwrap() != vp_holder {
            // EXPECTED ALWAYS
            let error = Errors::security_new("Invalid holder, it does not match");
            error!("{}", error.log());
            bail!(error);
        }
        info!("vp holder matches vpt subject & issuer");
        Ok(())
    }

    fn validate_issuer(&self, token: &TokenData<Value>, kid: &str) -> anyhow::Result<()> {
        info!("Validating issuer");

        let iss = get_opt_claim(&token.claims, vec!["iss"])?;
        let vc_iss_id = get_claim(&token.claims, vec!["vc", "issuer", "id"])?;

        check_iss(iss, kid)?;

        if vc_iss_id != kid {
            // VALIDATE IF ISSUER IS THE SAME AS KID
            let error = Errors::security_new("VCT token issuer & kid does not match");
            error!("{}", error.log());
            bail!(error);
        }
        info!("VC issuer & kid matches");
        Ok(())
    }

    fn validate_vc_id(&self, token: &TokenData<Value>) -> anyhow::Result<()> {
        info!("Validating VC id & JTI");

        let vc_id = get_claim(&token.claims, vec!["vc", "id"])?;
        let jti = get_opt_claim(&token.claims, vec!["jti"])?;

        if let Some(jti) = jti {
            if jti != vc_id {
                // VALIDATE ID MATCHES JTI
                let error = Errors::security_new("Invalid id, it does not match");
                error!("{}", error.log());
                bail!(error);
            }
            info!("VCT jti & VC id match");
        }

        Ok(())
    }

    fn validate_valid_from(&self, token: &TokenData<Value>) -> anyhow::Result<()> {
        info!("Validating issuance date");

        let valid_from = get_opt_claim(&token.claims, vec!["vc", "validFrom"])?;

        if let Some(valid_from) = valid_from {
            let date = DateTime::parse_from_rfc3339(&valid_from).map_err(|e| {
                let error = Errors::security_new(&format!(
                    "wrong format for field valid_from : {}",
                    e.to_string()
                ));
                error!("{}", error.log());
                error
            })?;
            if date > Utc::now() {
                let error = Errors::security_new("VC is not valid yet");
                error!("{}", error.log());
                bail!(error)
            }
            info!("VC has started its validity period correct");
        }

        Ok(())
    }

    fn validate_valid_until(&self, token: &TokenData<Value>) -> anyhow::Result<()> {
        info!("Validating expiration date");

        let valid_until = get_opt_claim(&token.claims, vec!["vc", "validUntil"])?;

        if let Some(valid_until) = valid_until {
            let date = DateTime::parse_from_rfc3339(&valid_until).map_err(|e| {
                let error = Errors::security_new(&format!(
                    "wrong format for field valid_until : {}",
                    e.to_string()
                ));
                error!("{}", error.log());
                error
            })?;
            if Utc::now() > date {
                let error = Errors::security_new("VC has expired");
                error!("{}", error.log());
                bail!(error)
            }
            info!("VC has not expired yet");
        }

        Ok(())
    }

    fn retrieve_vcs(&self, token: TokenData<Value>) -> anyhow::Result<Vec<String>> {
        info!("Retrieving VCs");
        let vcs: Vec<String> = serde_json::from_value(
            token.claims["vp"]["verifiableCredential"].clone(),
        )
        .map_err(|e| {
            let error = Errors::format_new(
                BadFormat::Received,
                &format!(
                    "VPT does not contain the 'verifiableCredential' field -> {}",
                    e.to_string()
                ),
            );
            error!("{}", error.log());
            error
        })?;

        Ok(vcs)
    }
}

fn check_iss(iss: Option<String>, kid: &str) -> anyhow::Result<()> {
    if let Some(iss) = iss {
        if iss != kid {
            let error = Errors::security_new("VPT token issuer & kid does not match");
            error!("{}", error.log());
            bail!(error);
        }
        info!("VPT issuer & kid matches");
    }
    Ok(())
}
