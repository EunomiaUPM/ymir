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

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jsonwebtoken::TokenData;
use serde_json::Value;
use tracing::info;
use urlencoding::encode;

use super::super::VerifierTrait;
use super::config::{BasicVerifierConfig, BasicVerifierConfigTrait};
use crate::config::traits::{HostsConfigTrait, VcConfigTrait};
use crate::config::types::HostType;
use crate::data::entities::recv_interaction;
use crate::data::entities::recv_verification::{Model, NewModel};
use crate::errors::{Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::types::errors::BadFormat;
use crate::types::gnap::ApprovedCallbackBody;
use crate::types::http::Body;
use crate::types::vcs::{VPDef, W3cDataModelVersion};
use crate::utils::{
    get_claim, get_opt_claim, json_headers, parse_to_string, parse_to_value, validate_token
};

pub struct BasicVerifierService {
    client: Arc<dyn ClientTrait>,
    config: BasicVerifierConfig
}

impl BasicVerifierService {
    pub fn new(client: Arc<dyn ClientTrait>, config: BasicVerifierConfig) -> BasicVerifierService {
        BasicVerifierService { client, config }
    }
}

#[async_trait]
impl VerifierTrait for BasicVerifierService {
    fn start_vp(&self, id: &str) -> Outcome<NewModel> {
        info!("Managing OIDC4VP");
        let host_url = self.config.get_host(HostType::Http);
        let host_url = match self.config.is_local() {
            true => host_url.replace("127.0.0.1", "host.docker.internal"),
            false => host_url
        };

        let client_id = format!("{}{}/verifier/verify", host_url, self.config.get_api_path(),);
        let requested_vcs = self.config.get_requested_vcs();
        if requested_vcs.is_empty() {
            return Err(Errors::unauthorized("Unable to verify following oidc4vp", None));
        }

        let mut vcs = vec![];

        for vc in requested_vcs {
            vcs.push(vc.name())
        }

        let vc_type = parse_to_string(&vcs)?;
        let new_verification_model = NewModel { id: id.to_string(), audience: client_id, vc_type };

        Ok(new_verification_model)
    }

    fn generate_verification_uri(&self, model: &Model) -> String {
        info!("Generating verification exchange URI");

        let host_url = self.config.get_host(HostType::Http);
        let host_url = format!("{}{}/verifier", host_url, self.config.get_api_path());
        let host_url = match self.config.is_local() {
            true => host_url.replace("127.0.0.1", "host.docker.internal"),
            false => host_url
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

    fn generate_vpd(&self, ver_model: &Model) -> Outcome<VPDef> {
        info!("Generating an vp definition");
        let model = self
            .config
            .get_w3c_data_model()
            .ok_or_else(|| Errors::not_active("W3c data model", None))?;
        Ok(VPDef::new(&ver_model.id, &ver_model.vc_type, model))
    }

    async fn verify_all(&self, ver_model: &mut Model, vp_token: &str) -> Outcome<()> {
        info!("Verifying all");

        let (vcs, holder) = self.verify_vp(ver_model, &vp_token).await?;
        for vc in vcs {
            self.verify_vc(&vc, &holder).await?;
        }
        info!("VP & VC Validated successfully");

        Ok(())
    }

    async fn verify_vp(&self, model: &mut Model, vp_token: &str) -> Outcome<(Vec<String>, String)> {
        info!("Verifying vp");

        model.vpt = Some(vp_token.to_string());
        let (token, kid) =
            validate_token(vp_token, Some(&model.state), self.client.clone()).await?;
        self.validate_nonce(model, &token)?;
        self.validate_vp_subject(model, &token, &kid)?;
        self.validate_vp_id(model, &token)?;
        self.validate_holder(model, &token)?;

        info!("VP Verification successful");
        let vcs = self.retrieve_vcs(token)?;

        Ok((vcs, kid))
    }

    async fn verify_vc(&self, vc_token: &str, holder: &str) -> Outcome<()> {
        info!("Verifying vc");

        let (token, kid) = validate_token(vc_token, None, self.client.clone()).await?;
        let model = self
            .config
            .get_w3c_data_model()
            .ok_or_else(|| Errors::not_active("W3c data model", None))?;
        self.validate_issuer(&token, &kid, &model)?;
        self.validate_vc_id(&token, &model)?;
        self.validate_vc_sub(&token, holder, &model)?;

        // if issuers_list.contains(kid) {
        //     // TODO
        //     error!("VCT issuer is not on the trusted issuers list");
        //     bail!("VCT issuer is not on the trusted issuers list");
        // }
        // info!("VCT issuer is on the trusted issuers list");

        self.validate_valid_from(&token, &model)?;
        self.validate_valid_until(&token, &model)?;

        info!("VC Verification successful");

        Ok(())
    }

    fn validate_nonce(&self, model: &Model, token: &TokenData<Value>) -> Outcome<()> {
        info!("Validating nonce");

        let nonce = get_claim(&token.claims, &["nonce"])?;

        if model.nonce != nonce {
            return Err(Errors::security("Invalid nonce, it does not match", None));
        }
        info!("VPT Nonce matches");
        Ok(())
    }

    fn validate_vp_subject(
        &self,
        model: &mut Model,
        token: &TokenData<Value>,
        kid: &str
    ) -> Outcome<()> {
        info!("Validating subject");

        let sub = get_opt_claim(&token.claims, &["sub"])?;
        let iss = get_opt_claim(&token.claims, &["iss"])?;

        if let Some(sub) = sub {
            if sub != kid {
                return Err(Errors::security("VPT token subject & kid does not match", None));
            }
            info!("VPT subject & kid matches");
        };

        check_iss(iss, kid)?;

        model.holder = Some(kid.to_string());
        Ok(())
    }

    fn validate_vc_sub(
        &self,
        token: &TokenData<Value>,
        holder: &str,
        model: &W3cDataModelVersion
    ) -> Outcome<()> {
        info!("Validating VC subject");

        let cred_sub_id = match model {
            W3cDataModelVersion::V1 => {
                get_claim(&token.claims, &["vc", "CredentialSubject", "id"])?
            }
            W3cDataModelVersion::V2 => get_claim(&token.claims, &["CredentialSubject", "id"])?
        };

        let sub = get_opt_claim(&token.claims, &["sub"])?;

        if let Some(sub) = sub {
            if sub != holder {
                return Err(Errors::security(
                    "VCT token sub, credential subject & VP Holder do not match",
                    None
                ));
            }
            info!("Sub & Holder match");
        };

        if holder != cred_sub_id {
            return Err(Errors::security(
                "VCT token sub, credential subject & VP Holder do not match",
                None
            ));
        }
        info!("Vc Holder & Holder match");
        Ok(())
    }

    fn validate_vp_id(&self, model: &Model, token: &TokenData<Value>) -> Outcome<()> {
        info!("Validating vp id");

        let vp_id = get_claim(&token.claims, &["vp", "id"])?;

        if model.id != vp_id {
            // VALIDATE ID MATCHES JTI
            return Err(Errors::security("Invalid id, it does not match", None));
        }
        info!("Exchange is valid");
        Ok(())
    }

    fn validate_holder(&self, model: &Model, token: &TokenData<Value>) -> Outcome<()> {
        info!("Validating holder");

        let vp_holder = get_claim(&token.claims, &["vp", "holder"])?;

        if model.holder.clone().unwrap() != vp_holder {
            // EXPECTED ALWAYS
            return Err(Errors::security("Invalid holder, it does not match", None));
        }
        info!("vp holder matches vpt subject & issuer");
        Ok(())
    }

    fn validate_issuer(
        &self,
        token: &TokenData<Value>,
        kid: &str,
        model: &W3cDataModelVersion
    ) -> Outcome<()> {
        info!("Validating issuer");

        let vc_iss_id = match model {
            W3cDataModelVersion::V1 => get_claim(&token.claims, &["vc", "issuer", "id"])?,
            W3cDataModelVersion::V2 => get_claim(&token.claims, &["issuer", "id"])?
        };
        let iss = get_opt_claim(&token.claims, &["iss"])?;

        check_iss(iss, kid)?;

        if vc_iss_id != kid {
            // VALIDATE IF ISSUER IS THE SAME AS KID
            return Err(Errors::security("VCT token issuer & kid does not match", None));
        }
        info!("VC issuer & kid matches");
        Ok(())
    }

    fn validate_vc_id(&self, token: &TokenData<Value>, model: &W3cDataModelVersion) -> Outcome<()> {
        info!("Validating VC id & JTI");

        let vc_id = match model {
            W3cDataModelVersion::V1 => get_claim(&token.claims, &["vc", "id"])?,
            W3cDataModelVersion::V2 => get_claim(&token.claims, &["id"])?
        };
        let jti = get_opt_claim(&token.claims, &["jti"])?;

        if let Some(jti) = jti {
            if jti != vc_id {
                // VALIDATE ID MATCHES JTI
                return Err(Errors::security("Invalid id, it does not match", None));
            }
            info!("VCT jti & VC id match");
        }

        Ok(())
    }

    fn validate_valid_from(
        &self,
        token: &TokenData<Value>,
        model: &W3cDataModelVersion
    ) -> Outcome<()> {
        info!("Validating issuance date");

        let valid_from = match model {
            W3cDataModelVersion::V1 => get_opt_claim(&token.claims, &["vc", "validFrom"])?,
            W3cDataModelVersion::V2 => get_opt_claim(&token.claims, &["validFrom"])?
        };

        if let Some(valid_from) = valid_from {
            let date = DateTime::parse_from_rfc3339(&valid_from).map_err(|e| {
                Errors::security("wrong format for field valid_from", Some(anyhow::Error::from(e)))
            })?;
            if date > Utc::now() {
                return Err(Errors::security("VC is not valid yet", None));
            }
            info!("VC has started its validity period");
        }

        Ok(())
    }

    fn validate_valid_until(
        &self,
        token: &TokenData<Value>,
        model: &W3cDataModelVersion
    ) -> Outcome<()> {
        info!("Validating expiration date");

        let valid_until = match model {
            W3cDataModelVersion::V1 => get_opt_claim(&token.claims, &["vc", "validUntil"])?,
            W3cDataModelVersion::V2 => get_opt_claim(&token.claims, &["validUntil"])?
        };

        if let Some(valid_until) = valid_until {
            let date = DateTime::parse_from_rfc3339(&valid_until).map_err(|e| {
                Errors::security("wrong format for field valid_until", Some(anyhow::Error::from(e)))
            })?;
            if Utc::now() > date {
                return Err(Errors::security("VC has expired", None));
            }
            info!("VC has not expired yet");
        }

        Ok(())
    }

    fn retrieve_vcs(&self, token: TokenData<Value>) -> Outcome<Vec<String>> {
        info!("Retrieving VCs");
        let vcs: Vec<String> = serde_json::from_value(
            token.claims["vp"]["verifiableCredential"].clone()
        )
        .map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "VPT does not contain de vc field",
                Some(anyhow::Error::from(e))
            )
        })?;

        Ok(vcs)
    }
    async fn end_verification(&self, model: &recv_interaction::Model) -> Outcome<Option<String>> {
        info!("Ending verification");

        if model.method == "redirect" {
            let redirect_uri = format!(
                "{}?hash={}&interact_ref={}",
                model.uri, model.hash, model.interact_ref
            );
            Ok(Some(redirect_uri))
        } else if model.method == "push" {
            let url = model.uri.clone();

            let headers = json_headers();

            let body = ApprovedCallbackBody {
                interact_ref: model.interact_ref.clone(),
                hash: model.hash.clone()
            };
            let body = parse_to_value(&body)?;
            self.client.post(&url, Some(headers), Body::Json(body)).await?;

            Ok(None)
        } else {
            Err(Errors::not_impl(format!("Interact method '{}'", model.method), None))
        }
    }
}

fn check_iss(iss: Option<String>, kid: &str) -> Outcome<()> {
    if let Some(iss) = iss {
        if iss != kid {
            return Err(Errors::security("VPT token issuer & kid does not match", None));
        }
        info!("VPT issuer & kid matches");
    }
    Ok(())
}
