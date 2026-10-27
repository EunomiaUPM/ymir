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

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde_json::{Map, Value, json};
use tokio::sync::RwLock;
use tracing::info;
use urlencoding::encode;

use super::super::VerifierTrait;
use super::VerifierConfig;
use crate::capabilities::{Signer, Verifier};
use crate::config::traits::HostsConfigTrait;
use crate::config::types::HostType;
use crate::data::entities::received::verification::{Model, Plan};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::http::routes::{base, fill, verifier};
use crate::services::vault::{VaultService, VaultTrait};
use crate::services::verifier::validators::{
    validate_nonce, validate_valid_from, validate_valid_until, validate_vc_id, validate_vc_issuer,
    validate_vc_sub, validate_vp_holder,
};
use crate::types::dids::Did;
use crate::types::jwt::{Jwt, JwtClaims, JwtHeaderTrait};
use crate::types::keys::{Alg, PrivateKey, SigningCtx};
use crate::types::secrets::PemHelper;
use crate::types::vcs::{VPDef, VcFormat, VcPayload};
use crate::types::verification::request_object::SELF_ISSUED_AUDIENCE;
use crate::types::verification::{CredentialQuery, DECENTRALIZED_IDENTIFIER_PREFIX, DIRECT_POST, DcqlQuery, REDIRECT_URI_PREFIX, RequestObjectPayload, RequestPassing, VerificationStatus};
use crate::types::vps::VpPayload;
use crate::types::wallet::Identity;
use crate::utils::Verified;

/// How long a served Request Object stays valid.
const REQUEST_OBJECT_TTL_SECS: i64 = 300;

/// Verifiable Presentation verification service following OpenID for Verifiable Presentations
/// **1.0**.
///
/// By default the request goes by reference (§5.10): the `openid4vp://` URI carries the
/// Client Identifier (`decentralized_identifier:<did>`, §5.9.3) and a `request_uri` where the
/// wallet fetches a Request Object signed with the Verifier's DID. With
/// [`RequestPassing::ByValue`] every parameter travels in the URI, unsigned, under the
/// `redirect_uri:` prefix. Either way the request holds a DCQL query with one `jwt_vc_json`
/// credential per requested type, the answer comes by `direct_post` (§8.2), and its `vp_token`
/// maps each credential query id to its presentations (§8.1).
pub struct VerifierService {
    config: VerifierConfig,
    vault: Arc<VaultService>,
    identity: Arc<RwLock<Identity>>,
}

impl VerifierService {
    pub fn new(
        config: VerifierConfig,
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
impl VerifierTrait for VerifierService {
    /// The plan's audience is the response URI without the state, which [`Plan`]'s conversion
    /// appends: once stored, `audience` is the verification's response URI.
    fn build_vp_plan(&self, grant_id: &str) -> Outcome<Plan> {
        info!("Managing OpenID4VP 1.0");

        let requested_vcs = self.config.get_requested_vcs();
        if requested_vcs.is_empty() {
            return Err(Errors::unauthorized(
                "Unable to verify following OpenID4VP 1.0: no credentials requested",
                None,
            ));
        }

        Ok(Plan {
            grant_id: grant_id.to_string(),
            audience: format!("{}{}", self.verifier_url(), base(verifier::VERIFY)),
            vc_type: requested_vcs.to_vec(),
        })
    }

    async fn generate_verification_uri(&self, model: &Model) -> Outcome<String> {
        info!("Generating OpenID4VP 1.0 authorization request");

        let client_id = self.client_id(model).await;
        let uri = match self.config.get_request_passing() {
            RequestPassing::ByReference => {
                let request_uri = format!(
                    "{}{}",
                    self.verifier_url(),
                    fill(verifier::REQUEST, &model.state)
                );
                format!(
                    "openid4vp://authorize\
                     ?client_id={}\
                     &request_uri={}\
                     &request_uri_method=post",
                    encode(&client_id),
                    encode(&request_uri),
                )
            }
            RequestPassing::ByValue => format!(
                "openid4vp://authorize\
                 ?response_type=vp_token\
                 &client_id={}\
                 &response_mode={}\
                 &response_uri={}\
                 &nonce={}\
                 &state={}\
                 &dcql_query={}\
                 &client_metadata={}",
                encode(&client_id),
                DIRECT_POST,
                encode(&model.audience),
                encode(&model.nonce),
                encode(&model.state),
                encode(&serde_json::to_string(&dcql_query(model))?),
                encode(&client_metadata().to_string()),
            ),
        };
        info!("Uri generated successfully: {uri}");
        Ok(uri)
    }

    async fn request_object(&self, model: &Model, wallet_nonce: Option<String>) -> Outcome<String> {
        info!("Serving OpenID4VP 1.0 request object");

        if self.config.get_request_passing() != RequestPassing::ByReference {
            return Err(Errors::not_impl(
                "This verifier passes its requests by value",
                None,
            ));
        }
        if model.status != VerificationStatus::Pending {
            return Err(Errors::forbidden(
                "The verification is no longer waiting for a presentation",
                None,
            ));
        }

        let lock = self.identity.read().await;
        let did = lock.did().clone();
        let key_ref = lock.key_ref().clone();
        drop(lock);

        let client_id = format!("{}{}", DECENTRALIZED_IDENTIFIER_PREFIX, did.id());
        let now = Utc::now();
        let claims = JwtClaims::new(RequestObjectPayload {
            client_id: client_id.clone(),
            response_type: "vp_token".to_string(),
            response_mode: DIRECT_POST.to_string(),
            response_uri: model.audience.clone(),
            state: Some(model.state.clone()),
            dcql_query: dcql_query(model),
            client_metadata: Some(client_metadata()),
            wallet_nonce,
        })
        .with_iss(&client_id)
        .with_aud(SELF_ISSUED_AUDIENCE)
        .with_nonce(&model.nonce)
        .with_iat(now)
        .with_exp(now + Duration::seconds(REQUEST_OBJECT_TTL_SECS));

        let pem_helper: PemHelper = self.vault.read(None, key_ref.internal()).await?;
        let key = PrivateKey::try_from(pem_helper)?;
        let sig_ctx = SigningCtx::new(did, key, key_ref.fragment().to_string());

        let jwt = Signer::sign_enveloped(&sig_ctx, &claims)?;
        Ok(jwt.as_str().to_string())
    }

    /// OpenID4VP 1.0 has no Presentation Definition: the request carries a DCQL query.
    fn generate_vpd(&self, _verification: &Model) -> Outcome<VPDef> {
        Err(Errors::not_impl(
            "OpenID4VP 1.0 requests credentials with DCQL, not Presentation Definitions",
            None,
        ))
    }

    /// Every presentation of the `vp_token` is verified; the first one is returned (they all
    /// come from the same holder).
    async fn verify_all(&self, model: &mut Model, vp_token: &str) -> Outcome<Verified<Jwt<VpPayload>>> {
        info!("Verifying all");

        let result = self.verify_vp_token(model, vp_token).await;

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
    /// `<host><api path>/verifier`.
    fn verifier_url(&self) -> String {
        format!(
            "{}{}{}",
            self.config.get_host(HostType::Http),
            self.config.get_api_path(),
            verifier::PREFIX
        )
    }

    /// The Client Identifier of the request for `model`, prefix included: the Verifier's DID
    /// when passed by reference, its response URI when by value. It is the `aud` of every
    /// presentation (B.1.3.1.2).
    async fn client_id(&self, model: &Model) -> String {
        match self.config.get_request_passing() {
            RequestPassing::ByReference => {
                let lock = self.identity.read().await;
                format!("{}{}", DECENTRALIZED_IDENTIFIER_PREFIX, lock.did().id())
            }
            RequestPassing::ByValue => format!("{}{}", REDIRECT_URI_PREFIX, model.audience),
        }
    }

    /// The whole `vp_token`: only requested queries, each answered and every presentation
    /// verified. Returns the first presentation.
    async fn verify_vp_token(
        &self,
        model: &mut Model,
        vp_token: &str,
    ) -> Outcome<Verified<Jwt<VpPayload>>> {
        model.vpt = Some(vp_token.to_string());
        let client_id = self.client_id(model).await;
        let query = dcql_query(model);
        let presentations = parse_vp_token(vp_token)?;
        check_only_requested(&query, &presentations)?;

        let mut vps: Vec<Verified<Jwt<VpPayload>>> = Vec::new();
        for credential in &query.credentials {
            for presentation in answers_to(credential, &presentations)? {
                let vp = self
                    .verify_presentation(model, &client_id, credential, presentation)
                    .await?;
                check_same_holder(vps.first(), &vp)?;
                vps.push(vp);
            }
        }

        let first = vps
            .into_iter()
            .next()
            .ok_or_else(|| Errors::security("vp_token holds no presentation", None))?;
        model.holder = Some(first.kid()?.did().id().to_string());
        Ok(first)
    }

    /// One presentation answering `credential`: the VP, its credentials, and one of them of the
    /// requested type.
    async fn verify_presentation(
        &self,
        model: &mut Model,
        client_id: &str,
        credential: &CredentialQuery,
        presentation: &str,
    ) -> Outcome<Verified<Jwt<VpPayload>>> {
        let vp = Jwt::<VpPayload>::parse(presentation)?;
        let vp = self.verify_vp(model, client_id, vp).await?;
        let holder = vp.kid()?.did().clone();

        let mut answers = false;
        for vc in vp.claims().vp_doc().credential_jwts()? {
            let types = self.verify_vc(&vc, &holder).await?;
            answers |= credential.matches_types(&types);
            model.vcs.push(vc);
        }
        if !answers {
            return Err(no_matching_credential(credential));
        }
        Ok(vp)
    }

    /// One presentation: signed by its holder for this Client Identifier (`aud`) and this
    /// request (`nonce`).
    async fn verify_vp(
        &self,
        model: &Model,
        client_id: &str,
        vp: Jwt<VpPayload>,
    ) -> Outcome<Verified<Jwt<VpPayload>>> {
        info!("Verifying vp");

        let jwt_v = Verifier::verify_enveloped(vp, Some(client_id.to_string())).await?;

        validate_vp_holder(&jwt_v)?;
        validate_nonce(&jwt_v, model)?;

        info!("VP verification successful");
        Ok(jwt_v)
    }

    /// One credential of a presentation. Returns its types, to match against the query.
    async fn verify_vc(&self, vc_token: &str, holder_did: &Did) -> Outcome<Vec<String>> {
        info!("Verifying vc");

        let vc = Jwt::<VcPayload>::parse(vc_token)?;
        let vc = Verifier::verify_enveloped(vc, None).await?;

        validate_vc_issuer(&vc)?;
        validate_vc_id(&vc)?;
        validate_vc_sub(&vc, holder_did)?;
        // TODO: trusted-issuer list once available (DCQL trusted_authorities)
        validate_valid_from(&vc)?;
        validate_valid_until(&vc)?;

        info!("VC verification successful");
        Ok(vc.claims().vc_doc().r#type.as_slice().to_vec())
    }
}

/// The DCQL query of a verification: one credential query per requested type.
fn dcql_query(model: &Model) -> DcqlQuery {
    DcqlQuery::for_types(&model.vc_type)
}

/// The Verifier's metadata sent with the request (§5.1, §11.1): the credential formats and
/// signature algorithms it accepts.
fn client_metadata() -> Value {
    let mut formats = Map::new();
    formats.insert(
        VcFormat::JwtVcJson.to_string(),
        json!({ "alg_values": Alg::supported() }),
    );
    json!({ "vp_formats_supported": formats })
}

/// The `vp_token` of OpenID4VP 1.0: a JSON object from credential query id to the array of
/// presentations answering it (§8.1); `jwt_vc_json` presentations are JWT strings.
fn parse_vp_token(vp_token: &str) -> Outcome<HashMap<String, Vec<String>>> {
    serde_json::from_str(vp_token).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            format!("vp_token is not an object of presentation arrays by query id: {e}"),
            None,
        )
    })
}

fn no_matching_credential(credential: &CredentialQuery) -> Errors {
    Errors::security(
        format!(
            "presentation for '{}' holds no credential of the requested type",
            credential.id
        ),
        None,
    )
}

/// Rejects a `vp_token` that answers a query nobody asked for.
fn check_only_requested(
    query: &DcqlQuery,
    presentations: &HashMap<String, Vec<String>>,
) -> Outcome<()> {
    match presentations.keys().find(|id| query.query(id).is_none()) {
        Some(id) => Err(Errors::security(
            format!("vp_token answers credential query '{id}', which was not requested"),
            None,
        )),
        None => Ok(()),
    }
}

/// The presentations answering `credential`: required (there are no credential_sets, §6.4.2),
/// and only one unless the query allows `multiple` (§8.1).
fn answers_to<'a>(
    credential: &CredentialQuery,
    presentations: &'a HashMap<String, Vec<String>>,
) -> Outcome<&'a [String]> {
    let vps = presentations
        .get(&credential.id)
        .filter(|vps| !vps.is_empty())
        .ok_or_else(|| {
            Errors::security(
                format!("vp_token lacks credential query '{}'", credential.id),
                None,
            )
        })?;
    if !credential.multiple && vps.len() > 1 {
        return Err(Errors::security(
            format!(
                "credential query '{}' allows one presentation, got {}",
                credential.id,
                vps.len()
            ),
            None,
        ));
    }
    Ok(vps)
}

/// Every presentation must come from the same holder: in the dataspace a participant is one DID.
fn check_same_holder(
    first: Option<&Verified<Jwt<VpPayload>>>,
    vp: &Verified<Jwt<VpPayload>>,
) -> Outcome<()> {
    if let Some(first) = first {
        if first.kid()?.did().id() != vp.kid()?.did().id() {
            return Err(Errors::security(
                "presentations come from different holders",
                None,
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{client_metadata, parse_vp_token};

    /// The token maps query ids to arrays of presentations; a bare JWT (draft 20) is refused.
    #[test]
    fn parses_the_vp_token_object() {
        let parsed = parse_vp_token(r#"{"DataSpaceParticipant":["a.b.c"]}"#).unwrap();
        assert_eq!(parsed["DataSpaceParticipant"], vec!["a.b.c".to_string()]);
        assert!(parse_vp_token("a.b.c").is_err());
    }

    /// The metadata names `jwt_vc_json` and its algorithms by their JOSE identifiers.
    #[test]
    fn client_metadata_lists_formats_and_algorithms() {
        let metadata = client_metadata();
        let algs = &metadata["vp_formats_supported"]["jwt_vc_json"]["alg_values"];
        assert!(algs.as_array().unwrap().iter().any(|a| a == "EdDSA"));
    }
}
