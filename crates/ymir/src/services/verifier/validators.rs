use std::str::FromStr;

use chrono::Utc;
use serde_json::Value;
use tracing::info;
use crate::data::entities::received::verification::Model;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::dids::Did;
use crate::types::jwt::{Jwt, JwtHeaderTrait};
use crate::types::vcs::vc_specs::gx_label::CompliantCredential;
use crate::types::vcs::{VcPayload, VcType};
use crate::types::verification::Rejection;
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

/// How a credential is tied to whoever presents it.
pub(in crate::services::verifier) enum Binding {
    /// Its subject is the holder.
    Subject,
    /// The holder issued it: the described participant is the issuing one (CD 4.0.0 PA1.1).
    Issuer,
    /// Its subject is its own node (`…#cs`); a legal person of the holder has to point to it.
    Linked(String),
}

/// A verified credential of a presentation, and how it is tied to the holder.
pub(in crate::services::verifier) struct PresentedVc {
    pub vc: Verified<Jwt<VcPayload>>,
    pub binding: Binding,
}

impl PresentedVc {
    pub fn types(&self) -> Vec<String> {
        self.vc.claims().vc_doc().r#type.as_slice().to_vec()
    }

    fn is(&self, wanted: &[VcType]) -> bool {
        self.types().iter().any(|t| {
            let Ok(t) = VcType::from_str(t);
            wanted.contains(&t)
        })
    }
}

/// Registration numbers and labels are their own nodes, as in Gaia-X.
const LINKED_TYPES: [VcType; 8] = [
    VcType::VatId,
    VcType::TaxId,
    VcType::LeiCode,
    VcType::Eori,
    VcType::Euid,
    VcType::LocalRegistrationNumber,
    VcType::GxLabel,
    VcType::ParticipantCompliance,
];

/// Whose the credential is: its subject is the holder, the holder issued it, or it is a
/// registration number or label whose own subject a legal person has to link (`check_bindings`).
pub(in crate::services::verifier) fn validate_vc_binding(jwt: Verified<Jwt<VcPayload>>, holder_did: &Did) -> Outcome<PresentedVc> {
    let subject = jwt
        .claims()
        .vc_doc()
        .credential_subject
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if subject == holder_did.id() {
        validate_vc_sub(&jwt, holder_did)?;
        return Ok(PresentedVc { vc: jwt, binding: Binding::Subject });
    }
    check_eq_opt(jwt.claims().sub(), &subject, "VCT sub & credentialSubject")?;
    let binding = if jwt.kid()?.did().id() == holder_did.id() {
        Binding::Issuer
    } else {
        Binding::Linked(subject)
    };
    let vc = PresentedVc { vc: jwt, binding };
    if matches!(vc.binding, Binding::Linked(_)) && !vc.is(&LINKED_TYPES) {
        return Err(Errors::security("VC credentialSubject does not match holder", None));
    }
    info!("VC bound to the holder");
    Ok(vc)
}

/// Every linked credential, tied to the holder by one of the holder's legal persons: a number by
/// its `gx:registrationNumber`, a label by the digest of the legal person it lists.
pub(in crate::services::verifier) fn check_bindings(vcs: &[PresentedVc]) -> Outcome<()> {
    let persons: Vec<&PresentedVc> = vcs
        .iter()
        .filter(|p| !matches!(p.binding, Binding::Linked(_)) && p.is(&[VcType::LegalPerson]))
        .collect();
    for vc in vcs {
        let Binding::Linked(subject) = &vc.binding else { continue };
        let tied = if vc.is(&[VcType::GxLabel, VcType::ParticipantCompliance]) {
            let digests = listed_digests(vc);
            persons.iter().any(|lp| {
                CompliantCredential::digest_of(lp.vc.as_str()).is_ok_and(|d| digests.contains(&d))
            })
        } else {
            persons.iter().any(|lp| {
                ids(lp.vc.claims().vc_doc().credential_subject.get("gx:registrationNumber"))
                    .contains(&subject.as_str())
            })
        };
        if !tied {
            return Err(Errors::security(
                format!("Credential {subject} is not tied to the holder by a legal person"),
                None,
            ));
        }
    }
    Ok(())
}

/// The `digestSRI` of each credential a label lists.
fn listed_digests(label: &PresentedVc) -> Vec<String> {
    let subject = &label.vc.claims().vc_doc().credential_subject;
    let entries = subject.get("gx:compliantCredentials").and_then(Value::as_array);
    entries
        .into_iter()
        .flatten()
        .filter_map(|e| e.get("digestSRI").or_else(|| e.get("gx:digestSRI")))
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

/// The `id`s a link holds: one object, or an array of them.
fn ids(link: Option<&Value>) -> Vec<&str> {
    fn one(v: &Value) -> Option<&str> {
        v.get("id").or_else(|| v.get("@id")).and_then(Value::as_str)
    }
    match link {
        Some(Value::Array(items)) => items.iter().filter_map(one).collect(),
        Some(v) => one(v).into_iter().collect(),
        None => Vec::new(),
    }
}

pub(in crate::services::verifier) fn validate_valid_from(jwt: &Verified<Jwt<VcPayload>>) -> Outcome<()> {
    info!("Validating issuance date");
    let claims = jwt.claims();
    if let Some(nbf) = claims.nbf() {
        is_active(nbf).map_err(|_| Rejection::NotYetValid.error("nbf"))?;
    }
    if let Some(iat) = claims.iat() {
        is_active(iat).map_err(|_| Rejection::NotYetValid.error("iat"))?;
    }
    if let Some(valid_from) = claims.vc_doc().valid_from {
        if valid_from > Utc::now() {
            return Err(Rejection::NotYetValid.error("validFrom"));
        }
        info!("VC has started its validity period");
    }
    Ok(())
}

pub(in crate::services::verifier) fn validate_valid_until(jwt: &Verified<Jwt<VcPayload>>) -> Outcome<()> {
    info!("Validating expiration date");
    let claims = jwt.claims();
    if let Some(exp) = claims.exp() {
        has_expired(exp).map_err(|_| Rejection::Expired.error("exp"))?;
    }
    if let Some(valid_until) = claims.vc_doc().valid_until {
        if Utc::now() > valid_until {
            return Err(Rejection::Expired.error("validUntil"));
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
