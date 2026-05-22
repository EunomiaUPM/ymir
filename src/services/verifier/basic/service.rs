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
use chrono::Utc;
use tracing::info;
use urlencoding::encode;

use super::super::VerifierTrait;
use super::config::{BasicVerifierConfig, BasicVerifierConfigTrait};
use crate::capabilities::Verifier;
use crate::config::traits::{HostsConfigTrait, VcConfigTrait};
use crate::config::types::HostType;
use crate::data::entities::recv_interaction;
use crate::data::entities::recv_verification::{Model, NewModel};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::types::gnap::ApprovedCallbackBody;
use crate::types::http::Body;
use crate::types::jwt::{Jwt, VCJwtClaimsV1, VCJwtClaimsV2, VPJwtClaims};
use crate::types::vcs::doc::VcDocument;
use crate::types::vcs::{VPDef, W3cDataModelVersion};
use crate::utils::json_headers;

pub struct BasicVerifierService {
    client: Arc<dyn ClientTrait>,
    config: BasicVerifierConfig,
}

impl BasicVerifierService {
    pub fn new(client: Arc<dyn ClientTrait>, config: BasicVerifierConfig) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl VerifierTrait for BasicVerifierService {
    fn start_vp(&self, id: &str) -> Outcome<NewModel> {
        info!("Managing OIDC4VP");

        let host_url = self.host_url();
        let client_id = format!("{}{}/verifier/verify", host_url, self.config.get_api_path());
        let requested_vcs = self.config.get_requested_vcs();
        if requested_vcs.is_empty() {
            return Err(Errors::unauthorized(
                "Unable to verify following oidc4vp",
                None,
            ));
        }

        Ok(NewModel {
            id: id.to_string(),
            audience: client_id,
            vc_type: requested_vcs.iter().map(|s| s.to_string()).collect(),
        })
    }

    fn generate_verification_uri(&self, model: &Model) -> String {
        info!("Generating verification exchange URI");

        let host_url = format!("{}{}/verifier", self.host_url(), self.config.get_api_path());
        let pd_uri = format!("{}/pd/{}", host_url, model.state);
        let response_uri = format!("{}/verify/{}", host_url, model.state);

        let uri = format!(
            "openid4vp://authorize\
             ?response_type=vp_token\
             &client_id={}\
             &response_mode=direct_post\
             &presentation_definition_uri={}\
             &client_id_scheme=redirect_uri\
             &nonce={}\
             &response_uri={}",
            encode(&model.audience),
            encode(&pd_uri),
            model.nonce,
            encode(&response_uri),
        );
        info!("Uri generated successfully: {uri}");
        uri
    }

    fn generate_vpd(&self, ver_model: &Model) -> Outcome<VPDef> {
        info!("Generating VP definition");
        let model = self
            .config
            .get_w3c_data_model()
            .ok_or_else(|| Errors::not_active("W3c data model", None))?;

        let vc_types: Vec<&str> = ver_model.vc_type.iter().map(String::as_str).collect();
        Ok(VPDef::new(&ver_model.id, &vc_types, model))
    }

    async fn verify_all(&self, ver_model: &mut Model, vp_token: &str) -> Outcome<()> {
        info!("Verifying all");
        let (vcs, holder) = self.verify_vp(ver_model, vp_token).await?;
        for vc in vcs {
            self.verify_vc(&vc, &holder).await?;
        }
        info!("VP & VC validated successfully");
        Ok(())
    }

    async fn end_verification(&self, model: &recv_interaction::Model) -> Outcome<Option<String>> {
        info!("Ending verification");
        match model.method.as_str() {
            "redirect" => {
                let uri = format!(
                    "{}?hash={}&interact_ref={}",
                    model.uri, model.hash, model.interact_ref
                );
                Ok(Some(uri))
            }
            "push" => {
                let body = ApprovedCallbackBody {
                    interact_ref: model.interact_ref.clone(),
                    hash: model.hash.clone(),
                };
                self.client
                    .post(&model.uri, Some(json_headers()), Body::json(&body)?)
                    .await?;
                Ok(None)
            }
            other => Err(Errors::not_impl(format!("Interact method '{other}'"), None)),
        }
    }
}

// ===== Internal helpers ======================================================

impl BasicVerifierService {
    fn host_url(&self) -> String {
        let host = self.config.get_host(HostType::Http);
        if self.config.is_local() {
            host.replace("127.0.0.1", "host.docker.internal")
        } else {
            host
        }
    }

    async fn verify_vp(&self, model: &mut Model, vp_token: &str) -> Outcome<(Vec<String>, String)> {
        info!("Verifying vp");
        model.vpt = Some(vp_token.to_string());

        let jwt = Jwt::parse(vp_token)?;
        Verifier::verify_enveloped(&jwt, Some(&model.audience)).await?;

        let kid = jwt.expect_kid()?.to_string();
        let claims: VPJwtClaims = jwt.claims()?;

        validate_nonce(model, &claims)?;
        validate_vp_subject(model, &claims, &kid)?;
        validate_vp_id(model, &claims)?;
        validate_holder(model, &claims)?;

        info!("VP verification successful");
        Ok((claims.vp.verifiable_credential, kid))
    }

    async fn verify_vc(&self, vc_token: &str, holder: &str) -> Outcome<()> {
        info!("Verifying vc");

        let jwt = Jwt::parse(vc_token)?;
        Verifier::verify_enveloped(&jwt, None).await?;

        let kid = jwt.expect_kid()?.to_string();
        let model = self
            .config
            .get_w3c_data_model()
            .ok_or_else(|| Errors::not_active("W3c data model", None))?;

        let (iss, sub, jti, vc) = parse_vc_claims(&jwt, &model)?;

        validate_vc_issuer(&vc, iss.as_deref(), &kid)?;
        validate_vc_id(&vc, jti.as_deref())?;
        validate_vc_sub(&vc, sub.as_deref(), holder)?;
        // TODO: trusted-issuer list once available
        validate_valid_from(&vc)?;
        validate_valid_until(&vc)?;

        info!("VC verification successful");
        Ok(())
    }
}

// ===== Free validators (pure logic, no `self`) ===============================

fn validate_nonce(model: &Model, claims: &VPJwtClaims) -> Outcome<()> {
    info!("Validating nonce");
    if model.nonce != claims.nonce {
        return Err(Errors::security("Invalid nonce, it does not match", None));
    }
    info!("VPT nonce matches");
    Ok(())
}

fn validate_vp_subject(model: &mut Model, claims: &VPJwtClaims, kid: &str) -> Outcome<()> {
    info!("Validating VP subject");
    check_eq_opt(claims.sub.as_deref(), kid, "VPT sub & kid")?;
    check_eq_opt(claims.iss.as_deref(), kid, "VPT iss & kid")?;
    model.holder = Some(kid.to_string());
    Ok(())
}

fn validate_vp_id(model: &Model, claims: &VPJwtClaims) -> Outcome<()> {
    info!("Validating vp id");
    if model.id != claims.vp.id {
        return Err(Errors::security("Invalid id, it does not match", None));
    }
    info!("Exchange is valid");
    Ok(())
}

fn validate_holder(model: &Model, claims: &VPJwtClaims) -> Outcome<()> {
    info!("Validating holder");
    let expected = model
        .holder
        .as_deref()
        .ok_or_else(|| Errors::security("Holder not set in model", None))?;
    let actual = claims
        .vp
        .holder
        .as_deref()
        .ok_or_else(|| Errors::format(BadFormat::Received, "vp.holder missing", None))?;
    if expected != actual {
        return Err(Errors::security("Invalid holder, it does not match", None));
    }
    info!("vp holder matches");
    Ok(())
}

fn validate_vc_issuer(vc: &VcDocument, iss: Option<&str>, kid: &str) -> Outcome<()> {
    info!("Validating VC issuer");
    check_eq_opt(iss, kid, "VCT iss & kid")?;
    if vc.issuer.id != kid {
        return Err(Errors::security(
            "VCT token issuer & kid does not match",
            None,
        ));
    }
    info!("VC issuer & kid match");
    Ok(())
}

fn validate_vc_id(vc: &VcDocument, jti: Option<&str>) -> Outcome<()> {
    info!("Validating VC id");
    if let Some(jti) = jti {
        if jti != vc.id {
            return Err(Errors::security("Invalid id, it does not match", None));
        }
        info!("VCT jti & VC id match");
    }
    Ok(())
}

fn validate_vc_sub(vc: &VcDocument, sub: Option<&str>, holder: &str) -> Outcome<()> {
    info!("Validating VC subject");
    let cred_sub = vc
        .credential_subject
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "credentialSubject.id missing or not a string",
                None,
            )
        })?;

    if let Some(sub) = sub {
        if sub != holder {
            return Err(Errors::security("VCT sub does not match holder", None));
        }
    }
    if cred_sub != holder {
        return Err(Errors::security(
            "VC credentialSubject does not match holder",
            None,
        ));
    }
    info!("VC subject & holder match");
    Ok(())
}

fn validate_valid_from(vc: &VcDocument) -> Outcome<()> {
    info!("Validating issuance date");
    if let Some(valid_from) = vc.valid_from {
        if valid_from > Utc::now() {
            return Err(Errors::security("VC is not valid yet", None));
        }
        info!("VC has started its validity period");
    }
    Ok(())
}

fn validate_valid_until(vc: &VcDocument) -> Outcome<()> {
    info!("Validating expiration date");
    if let Some(valid_until) = vc.valid_until {
        if Utc::now() > valid_until {
            return Err(Errors::security("VC has expired", None));
        }
        info!("VC has not expired yet");
    }
    Ok(())
}

fn check_eq_opt(actual: Option<&str>, expected: &str, ctx: &str) -> Outcome<()> {
    if let Some(a) = actual {
        if a != expected {
            return Err(Errors::security(format!("{ctx} does not match"), None));
        }
        info!("{ctx} match");
    }
    Ok(())
}

fn parse_vc_claims(
    jwt: &Jwt,
    model: &W3cDataModelVersion,
) -> Outcome<(Option<String>, Option<String>, Option<String>, VcDocument)> {
    match model {
        W3cDataModelVersion::V1 => {
            let c: VCJwtClaimsV1 = jwt.claims()?;
            Ok((c.iss, c.sub, c.jti, c.vc))
        }
        W3cDataModelVersion::V2 => {
            let c: VCJwtClaimsV2 = jwt.claims()?;
            Ok((c.iss, c.sub, c.jti, c.vc))
        }
    }
}
