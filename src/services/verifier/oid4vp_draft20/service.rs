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

use async_trait::async_trait;
use chrono::Utc;
use tracing::info;
use urlencoding::encode;

use super::super::VerifierTrait;
use super::VerifierConfig;
use crate::capabilities::{Did, Kid, Verifier};
use crate::config::traits::HostsConfigTrait;
use crate::config::types::HostType;
use crate::data::entities::received::verification::{Model, Plan};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::jwt::{Jwt, VCJwtClaims, VPJwtClaims};
use crate::types::vcs::{VPDef, W3cDataModelVersion};
use crate::types::verification::VerificationStatus;
use crate::utils::{has_expired, is_active};

pub struct VerifierService {
    config: VerifierConfig,
}

impl VerifierService {
    pub fn new(config: VerifierConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl VerifierTrait for VerifierService {
    fn build_vp_plan(&self, id: &str) -> Outcome<Plan> {
        info!("Managing OIDC4VP");

        let host_url = self.config.get_host(HostType::Http);
        let client_id = format!("{}{}/verifier/verify", host_url, self.config.get_api_path());
        let requested_vcs = self.config.get_requested_vcs();
        if requested_vcs.is_empty() {
            return Err(Errors::unauthorized(
                "Unable to verify following oidc4vp",
                None,
            ));
        }

        Ok(Plan {
            id: id.to_string(),
            audience: client_id,
            vc_type: requested_vcs.to_vec(),
        })
    }

    fn generate_verification_uri(&self, model: &Model) -> String {
        info!("Generating verification exchange URI");

        let host_url = format!(
            "{}{}/verifier",
            self.config.get_host(HostType::Http),
            self.config.get_api_path()
        );
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

    fn generate_vpd(&self, verification: &Model) -> Outcome<VPDef> {
        info!("Generating VP definition");

        Ok(VPDef::new(
            &verification.id,
            &verification.vc_type,
            W3cDataModelVersion::default(),
        ))
    }

    async fn verify_all(&self, model: &mut Model, vp_token: &str) -> Outcome<()> {
        info!("Verifying all");

        let result: Outcome<()> = async {
            let (vcs, holder_did) = self.verify_vp(model, vp_token).await?;

            for vc in vcs {
                self.verify_vc(&vc, &holder_did).await?;
                model.vcs.push(vc)
            }
            Ok(())
        }
            .await;

        model.ended_at = Some(Utc::now());
        model.status = match &result {
            Ok(()) => {
                info!("VP & VC validated successfully");
                VerificationStatus::Verified
            }
            Err(_) => VerificationStatus::Failed,
        };

        result
    }
}

// ===== Internal helpers ======================================================

impl VerifierService {
    async fn verify_vp(&self, model: &mut Model, vp_token: &str) -> Outcome<(Vec<String>, Did)> {
        info!("Verifying vp");
        model.vpt = Some(vp_token.to_string());

        let jwt = Jwt::parse(vp_token)?;
        let (holder_kid, claims) =
            Verifier::verify_enveloped::<VPJwtClaims>(&jwt, Some(&model.audience)).await?;

        validate_vp_holder(&claims, &holder_kid)?;
        model.holder = Some(holder_kid.did().id().to_string());
        validate_vp_id(&claims, model)?;
        validate_nonce(&claims, model)?;

        info!("VP verification successful");
        Ok((claims.vp.verifiable_credential, holder_kid.did().to_owned()))
    }

    async fn verify_vc(&self, vc_token: &str, holder_did: &Did) -> Outcome<()> {
        info!("Verifying vc");

        let jwt = Jwt::parse(vc_token)?;
        let (iss_kid, claims) = Verifier::verify_enveloped::<VCJwtClaims>(&jwt, None).await?;

        validate_vc_issuer(&claims, &iss_kid)?;
        validate_vc_id(&claims)?;
        validate_vc_sub(&claims, holder_did)?;
        // TODO: trusted-issuer list once available
        validate_valid_from(&claims)?;
        validate_valid_until(&claims)?;

        info!("VC verification successful");
        Ok(())
    }
}

// ===== Free validators (pure logic, no `self`) ===============================

fn validate_nonce(claims: &VPJwtClaims, model: &Model) -> Outcome<()> {
    info!("Validating nonce");
    if model.nonce != claims.nonce {
        return Err(Errors::security("Invalid nonce, it does not match", None));
    }
    info!("VPT nonce matches");
    Ok(())
}

fn validate_vp_holder(claims: &VPJwtClaims, holder_kid: &Kid) -> Outcome<()> {
    info!("Validating VP subject");
    check_eq_opt(
        claims.sub.as_deref(),
        holder_kid.did().id(),
        "VPT sub & kid",
    )?;
    check_eq_opt(
        claims.iss.as_deref(),
        holder_kid.did().id(),
        "VPT iss & kid",
    )?;
    check_eq_opt(
        claims.vp.holder.as_deref(),
        holder_kid.did().id(),
        "VP holder & kid",
    )?;
    Ok(())
}

fn validate_vp_id(claims: &VPJwtClaims, model: &Model) -> Outcome<()> {
    info!("Validating vp id");
    if model.id != claims.vp.id {
        return Err(Errors::security("Invalid id, it does not match", None));
    }
    info!("Exchange is valid");
    Ok(())
}

fn validate_vc_issuer(claims: &VCJwtClaims, issuer_did: &Kid) -> Outcome<()> {
    info!("Validating VC issuer");
    check_eq_opt(claims.iss(), issuer_did.did().id(), "VCT iss & kid")?;
    if claims.vc_doc().issuer.id() != issuer_did.did().id() {
        return Err(Errors::security(
            "VCT token issuer & kid does not match",
            None,
        ));
    }
    info!("VC issuer & kid match");
    Ok(())
}

fn validate_vc_id(claims: &VCJwtClaims) -> Outcome<()> {
    info!("Validating VC id");
    check_eq_opt(claims.jti(), &claims.vc_doc().id, "VCT jti and vc id")
}

fn validate_vc_sub(claims: &VCJwtClaims, holder_did: &Did) -> Outcome<()> {
    info!("Validating VC subject");
    let cred_sub_id = claims
        .vc_doc()
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

    check_eq_opt(
        claims.sub(),
        holder_did.id(),
        "VCT sub & and holder from vp",
    )?;
    if cred_sub_id != holder_did.id() {
        return Err(Errors::security(
            "VC credentialSubject does not match holder",
            None,
        ));
    }
    info!("VC subject & holder match");
    Ok(())
}

fn validate_valid_from(claims: &VCJwtClaims) -> Outcome<()> {
    info!("Validating issuance date");
    if let Some(nbf) = claims.nbf() {
        is_active(nbf)?;
    }
    if let Some(iat) = claims.iat() {
        is_active(iat)?;
    }
    if let Some(valid_from) = claims.vc_doc().valid_from {
        if valid_from > Utc::now() {
            return Err(Errors::security("VC is not valid yet", None));
        }
        info!("VC has started its validity period");
    }
    Ok(())
}

fn validate_valid_until(claims: &VCJwtClaims) -> Outcome<()> {
    info!("Validating expiration date");
    if let Some(exp) = claims.exp() {
        has_expired(exp)?;
    }
    if let Some(valid_until) = claims.vc_doc().valid_until {
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
