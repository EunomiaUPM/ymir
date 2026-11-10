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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant};

use chrono::Utc;
use tracing::{info, warn};

use crate::capabilities::Verifier;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::types::jwt::Jwt;
use crate::types::vcs::doc::{
    BITSTRING_STATUS_LIST, BITSTRING_STATUS_LIST_CREDENTIAL, BitstringStatusList,
    BitstringStatusListEntry, StatusPurpose, VcDocument,
};
use crate::types::vcs::{Bitstring, VcPayload};
use crate::types::verification::Rejection;
use crate::utils::{ResponseExt, Verified, has_expired, http_client};

pub const DEFAULT_STATUS_LIST_TTL_MS: u64 = 300_000;
const MAX_STATUS_LIST_TTL_MS: u64 = 3_600_000;
const CLOCK_SKEW_LEEWAY_SECS: i64 = 30;

struct CachedList {
    issuer: String,
    purpose: StatusPurpose,
    bits: Bitstring,
    expires_at: Instant,
}

static CACHE: LazyLock<RwLock<HashMap<String, CachedList>>> = LazyLock::new(Default::default);

/// Checks the Bitstring Status List entries of a verified VC (VC Bitstring Status List §3.2).
pub struct VcStatusCheck;

impl VcStatusCheck {
    pub async fn check(vc: &Verified<Jwt<VcPayload>>) -> Outcome<()> {
        let Some(statuses) = vc.claims().vc_doc().credential_status.as_ref() else {
            return Ok(());
        };
        let issuer = vc.signer()?.id().to_string();

        for status in statuses.iter() {
            let Some(entry) = status.as_bitstring() else {
                warn!("Unsupported credentialStatus, not checked");
                continue;
            };
            let rejection = match entry.status_purpose {
                StatusPurpose::Revocation => Rejection::Revoked,
                StatusPurpose::Suspension => Rejection::Suspended,
                purpose => {
                    info!("Status purpose {purpose} is not enforced");
                    continue;
                }
            };
            info!("Checking {} status of the VC", entry.status_purpose);
            if Self::is_set(entry, &issuer).await? {
                return Err(rejection.error("credentialStatus"));
            }
        }
        Ok(())
    }

    /// Whether the issuer has revoked or suspended `vc`. A list that cannot be read counts as
    /// neither: whoever verifies the credential decides.
    pub async fn is_withdrawn(vc: &Verified<Jwt<VcPayload>>) -> bool {
        match Self::check(vc).await {
            Ok(()) => false,
            Err(error) => matches!(
                Rejection::of(&error),
                Some(Rejection::Revoked | Rejection::Suspended)
            ),
        }
    }

    async fn is_set(entry: &BitstringStatusListEntry, issuer: &str) -> Outcome<bool> {
        if let Some(bit) = Self::cached(entry, issuer)? {
            return Ok(bit);
        }
        let list = Self::fetch(&entry.status_list_credential, issuer, entry.status_purpose).await?;
        let bit = list.bits.get(entry.index()?)?;

        let mut cache = CACHE.write().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        cache.retain(|_, l| l.expires_at > now);
        cache.insert(entry.status_list_credential.clone(), list);
        Ok(bit)
    }

    fn cached(entry: &BitstringStatusListEntry, issuer: &str) -> Outcome<Option<bool>> {
        let cache = CACHE.read().unwrap_or_else(|e| e.into_inner());
        match cache.get(&entry.status_list_credential) {
            Some(list) if list.expires_at > Instant::now() => {
                if list.issuer != issuer || list.purpose != entry.status_purpose {
                    return Err(Errors::security(
                        "Status list does not belong to the VC issuer or purpose",
                        None,
                    ));
                }
                Ok(Some(list.bits.get(entry.index()?)?))
            }
            _ => Ok(None),
        }
    }

    async fn fetch(url: &str, issuer: &str, purpose: StatusPurpose) -> Outcome<CachedList> {
        info!("Fetching status list {url}");
        let raw = http_client()
            .get(url, None)
            .await?
            .ensure_success()
            .await?
            .parse_text()
            .await?;
        let list = Verifier::verify_enveloped(Jwt::<VcPayload>::parse(raw.trim())?, None).await?;

        let claims = list.claims();
        let doc = claims.vc_doc();
        let signer = list.signer()?.id();
        if signer != issuer
            || doc.issuer_did() != issuer
            || claims.iss().is_some_and(|iss| iss != issuer)
        {
            return Err(Errors::security(
                "Status list is not signed by the VC issuer",
                None,
            ));
        }
        if !doc.r#type.iter().any(|t| t == BITSTRING_STATUS_LIST_CREDENTIAL) {
            return Err(Errors::security(
                "Status list is not a BitstringStatusListCredential",
                None,
            ));
        }

        let subject: BitstringStatusList = serde_json::from_value(doc.credential_subject.clone())
            .map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    "Invalid BitstringStatusList subject",
                    Some(Box::new(e)),
                )
            })?;
        if subject.r#type != BITSTRING_STATUS_LIST || subject.status_purpose != purpose {
            return Err(Errors::security(
                "Status list purpose does not match the VC entry",
                None,
            ));
        }

        let ttl_ms = subject
            .ttl
            .unwrap_or(DEFAULT_STATUS_LIST_TTL_MS)
            .min(MAX_STATUS_LIST_TTL_MS);
        if let Some(exp) = claims.exp() {
            has_expired(exp)?;
        }
        Self::check_fresh(claims.iat(), doc, ttl_ms)?;

        Ok(CachedList {
            issuer: issuer.to_string(),
            purpose,
            bits: Bitstring::decode(&subject.encoded_list)?,
            expires_at: Instant::now() + Duration::from_millis(ttl_ms),
        })
    }

    /// A list with `validUntil` is valid until then; without it, it must be issued within its ttl.
    fn check_fresh(iat: Option<i64>, doc: &VcDocument, ttl_ms: u64) -> Outcome<()> {
        let now = Utc::now().timestamp();
        let issued = iat.or(doc.valid_from.map(|t| t.timestamp()));

        if issued.is_some_and(|issued| issued > now + CLOCK_SKEW_LEEWAY_SECS) {
            return Err(Errors::security("Status list is not valid yet", None));
        }
        if let Some(until) = doc.valid_until {
            return match now - CLOCK_SKEW_LEEWAY_SECS <= until.timestamp() {
                true => Ok(()),
                false => Err(Errors::security("Status list has expired", None)),
            };
        }
        match issued {
            Some(issued) if now - issued <= (ttl_ms / 1000) as i64 + CLOCK_SKEW_LEEWAY_SECS => Ok(()),
            _ => Err(Errors::security("Status list is stale", None)),
        }
    }
}
