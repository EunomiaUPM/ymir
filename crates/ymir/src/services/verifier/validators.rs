use chrono::Utc;
use tracing::info;
use crate::data::entities::received::verification::Model;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::dids::Did;
use crate::types::jwt::{Jwt, JwtHeaderTrait};
use crate::types::vcs::VcPayload;
use crate::types::vps::VpPayload;
use crate::utils::{is_active, Verified, has_expired};

pub(in crate::services::verifier) fn validate_nonce(jwt: &Verified<Jwt<VpPayload>>, model: &Model) -> Outcome<()> {
    info!("Validating nonce");
    if let Some(nonce) = jwt.claims().nonce() {
        if &model.nonce != nonce {
            return Err(Errors::security("Invalid nonce, it does not match", None));
        }
    } else {
        return Err(Errors::security("Missing nonce in request", None));
    }

    info!("VPT nonce matches");
    Ok(())
}

pub(in crate::services::verifier) fn validate_vp_holder(jwt: &Verified<Jwt<VpPayload>>) -> Outcome<()> {
    info!("Validating VP subject");
    let holder = jwt.kid()?.did().id();
    let claims = jwt.claims();
    check_eq_opt(claims.sub(), holder, "VPT sub & kid")?;
    check_eq_opt(claims.iss(), holder, "VPT iss & kid")?;
    check_eq_opt(
        claims.vp_doc().holder.as_deref(),
        holder,
        "VP holder & kid",
    )?;
    Ok(())
}

pub(in crate::services::verifier) fn validate_vp_id(jwt: &Verified<Jwt<VpPayload>>, model: &Model) -> Outcome<()> {
    info!("Validating vp id");
    if model.id != jwt.claims().vp_doc().id {
        return Err(Errors::security("Invalid id, it does not match", None));
    }
    info!("Exchange is valid");
    Ok(())
}

pub(in crate::services::verifier) fn validate_vc_issuer(jwt: &Verified<Jwt<VcPayload>>) -> Outcome<()> {
    info!("Validating VC issuer");
    let claims = jwt.claims();
    let issuer = jwt.kid()?.did().id();
    check_eq_opt(claims.iss(), issuer, "VCT iss & kid")?;
    if claims.vc_doc().issuer.id() != issuer {
        return Err(Errors::security(
            "VCT token issuer & kid does not match",
            None,
        ));
    }
    info!("VC issuer & kid match");
    Ok(())
}

pub(in crate::services::verifier) fn validate_vc_id(jwt: &Verified<Jwt<VcPayload>>) -> Outcome<()> {
    info!("Validating VC id");
    let claims = jwt.claims();
    check_eq_opt(claims.jti(), &claims.vc_doc().id, "VCT jti and vc id")
}

pub(in crate::services::verifier) fn validate_vc_sub(jwt: &Verified<Jwt<VcPayload>>, holder_did: &Did) -> Outcome<()> {
    info!("Validating VC subject");
    let claims = jwt.claims();
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

pub(in crate::services::verifier) fn validate_valid_from(jwt: &Verified<Jwt<VcPayload>>) -> Outcome<()> {
    info!("Validating issuance date");
    let claims = jwt.claims();
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

pub(in crate::services::verifier) fn validate_valid_until(jwt: &Verified<Jwt<VcPayload>>) -> Outcome<()> {
    info!("Validating expiration date");
    let claims = jwt.claims();
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

pub(in crate::services::verifier) fn check_eq_opt(actual: Option<&str>, expected: &str, ctx: &str) -> Outcome<()> {
    if let Some(a) = actual {
        if a != expected {
            return Err(Errors::security(format!("{ctx} does not match"), None));
        }
        info!("{ctx} match");
    }
    Ok(())
}
