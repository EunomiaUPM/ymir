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
use crate::capabilities::Verifier;
use crate::config::traits::HostsConfigTrait;
use crate::config::types::HostType;
use crate::data::entities::received::verification::{Model, Plan};
use crate::errors::{Errors, Outcome};
use crate::http::routes::{base, fill, verifier};
use crate::services::verifier::validators::{validate_nonce, validate_valid_from, validate_valid_until, validate_vc_id, validate_vc_issuer, validate_vc_sub, validate_vp_holder, validate_vp_id};
use crate::types::dids::Did;
use crate::types::jwt::{Jwt, JwtHeaderTrait};
use crate::types::vcs::{VcPayload};
use crate::types::vps::{VpPayload};
use crate::types::vcs::{VPDef, W3cDataModelVersion};
use crate::types::verification::VerificationStatus;
use crate::utils::{Verified};

/// Verifiable Presentation verification service backed by an OpenID4VP implementation.
///
/// Follows the OpenID for Verifiable Presentations (OpenID4VP) **Draft 20** specification
/// to receive, parse, and evaluate incoming Verifiable Presentations against requirements
/// defined via the DIF Presentation Exchange.
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

        let requested_vcs = self.config.get_requested_vcs();
        if requested_vcs.is_empty() {
            return Err(Errors::unauthorized(
                "Unable to verify following oidc4vp",
                None,
            ));
        }

        Ok(Plan {
            id: id.to_string(),
            audience: format!("{}{}", self.verifier_url(), base(verifier::VERIFY)),
            vc_type: requested_vcs.to_vec(),
        })
    }

    async fn generate_verification_uri(&self, model: &Model) -> Outcome<String> {
        info!("Generating verification exchange URI");

        let pd_uri = format!("{}{}", self.verifier_url(), fill(verifier::PD, &model.state));
        let response_uri = format!("{}{}", self.verifier_url(), fill(verifier::VERIFY, &model.state));

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
        Ok(uri)
    }

    fn generate_vpd(&self, verification: &Model) -> Outcome<VPDef> {
        info!("Generating VP definition");

        Ok(VPDef::new(
            &verification.id,
            &verification.vc_type,
            W3cDataModelVersion::default(),
        ))
    }

    async fn verify_all(&self, model: &mut Model, vp_token: &str) -> Outcome<Verified<Jwt<VpPayload>>> {
        info!("Verifying all");

        let result: Outcome<Verified<Jwt<VpPayload>>> = async {
            let vp = Jwt::<VpPayload>::parse(vp_token)?;
            let vp = self.verify_vp(model, vp).await?;
            let holder = vp.kid()?.did();
            let vcs = vp.claims().vp_doc().credential_jwts()?;

            for vc in vcs {
                self.verify_vc(&vc, holder).await?;
                model.vcs.push(vc);
            }

            Ok(vp)
        }
            .await;

        model.ended_at = Some(Utc::now());
        model.status = match &result {
            Ok(_) => {
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
    fn verifier_url(&self) -> String {
        format!(
            "{}{}{}",
            self.config.get_host(HostType::Http),
            self.config.get_api_path(),
            verifier::PREFIX
        )
    }
    async fn verify_vp(&self, model: &mut Model, vp: Jwt<VpPayload>) -> Outcome<Verified<Jwt<VpPayload>>> {
        info!("Verifying vp");
        model.vpt = Some(vp.to_string());

        let vp = Verifier::verify_enveloped(vp, Some(model.audience.clone())).await?;

        validate_vp_holder(&vp)?;
        model.holder = Some(vp.kid()?.did().id().to_string());
        validate_vp_id(&vp, model)?;
        validate_nonce(&vp, model)?;

        info!("VP verification successful");
        Ok(vp)
    }

    async fn verify_vc(&self, vc_token: &str, holder_did: &Did) -> Outcome<()> {
        info!("Verifying vc");

        let vc = Jwt::<VcPayload>::parse(vc_token)?;
        let vc = Verifier::verify_enveloped(vc, None).await?;

        validate_vc_issuer(&vc)?;
        validate_vc_id(&vc)?;
        validate_vc_sub(&vc, holder_did)?;
        // TODO: trusted-issuer list once available
        validate_valid_from(&vc)?;
        validate_valid_until(&vc)?;

        info!("VC verification successful");
        Ok(())
    }
}

