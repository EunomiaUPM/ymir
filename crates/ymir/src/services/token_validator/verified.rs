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
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;

use super::OauthTokenValidatorTrait;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::services::client::ClientExt;
use crate::types::keys::{Alg, PublicKey};
use crate::types::oauth::UserInfo;
use crate::utils::{decode_url_safe_no_pad, http_client};

/// Seconds of clock skew tolerated on `exp` and `nbf`.
const LEEWAY_SECS: i64 = 30;

/// Header of an identity provider's JWT. Unlike [`Jwt`](crate::types::jwt::Jwt), whose `kid` is
/// a DID URL, here the `kid` is whatever the provider names its keys with.
#[derive(Deserialize)]
struct TokenHeader {
    alg: Alg,
    #[serde(default)]
    kid: Option<String>,
}

/// Reads the user from a JWT forwarded by the proxy in front of Keycloak, **verifying** it with
/// ymir's own keys and algorithms: its signature against the realm's keys, its expiry, its
/// issuer and, when configured, its audience.
///
/// The keys come from the realm's OpenID configuration (`{issuer}/.well-known/openid-configuration`
/// → `jwks_uri`) on first use; they are kept in memory and fetched again when a token is signed
/// with a key they do not hold (the realm rotated them). RSA (RS*/PS*) and Ed25519 keys only:
/// [`PublicKey`] has no ECDSA, and Keycloak signs with RS256 by default.
pub struct VerifiedTokenValidator {
    user_header: String,
    issuer: String,
    audience: Option<String>,
    keys: RwLock<Option<Vec<Value>>>,
}

impl VerifiedTokenValidator {
    /// Reads the user's token from `user_header` and accepts only the ones `issuer` signed.
    pub fn new(user_header: &str, issuer: impl Into<String>, audience: Option<String>) -> Self {
        Self {
            user_header: user_header.to_ascii_lowercase(),
            issuer: issuer.into(),
            audience,
            keys: RwLock::new(None),
        }
    }

    /// The realm's signing keys (JWKs): the ones in memory, or fetched again when `refresh`.
    async fn keys(&self, refresh: bool) -> Outcome<Vec<Value>> {
        if !refresh {
            if let Some(keys) = self.keys.read().await.as_ref() {
                return Ok(keys.clone());
            }
        }
        let discovery_url = format!(
            "{}/.well-known/openid-configuration",
            self.issuer.trim_end_matches('/')
        );
        let discovery: Value = http_client().get_json(&discovery_url, None).await?;
        let jwks_uri = discovery
            .get("jwks_uri")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                Errors::format(BadFormat::Received, "OpenID configuration without jwks_uri", None)
            })?;
        let jwks: Value = http_client().get_json(jwks_uri, None).await?;
        let keys = jwks
            .get("keys")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| Errors::format(BadFormat::Received, "JWKS without keys", None))?;
        *self.keys.write().await = Some(keys.clone());
        Ok(keys)
    }

    /// The signing key `kid` names (the only signing key when the token names none), fetching
    /// the keys again once if it is not among them.
    async fn public_key(&self, kid: Option<&str>) -> Outcome<PublicKey> {
        let pick = |keys: &[Value]| -> Option<Value> {
            let mut signing = keys
                .iter()
                .filter(|k| k.get("use").and_then(Value::as_str).is_none_or(|u| u == "sig"));
            match kid {
                Some(kid) => signing
                    .find(|k| k.get("kid").and_then(Value::as_str) == Some(kid))
                    .cloned(),
                None => signing.next().cloned(),
            }
        };
        let jwk = match pick(&self.keys(false).await?) {
            Some(jwk) => jwk,
            None => pick(&self.keys(true).await?)
                .ok_or_else(|| Errors::unauthorized("token signed with an unknown key", None))?,
        };
        PublicKey::parse_from_jwk(&jwk)
            .map_err(|e| Errors::unauthorized("unusable signing key", Some(Box::new(e))))
    }

    /// The claims checks: issuer, expiry, not-before and, when configured, audience.
    fn check_claims(&self, claims: &Value) -> Outcome<()> {
        let now = chrono::Utc::now().timestamp();
        if claims.get("iss").and_then(Value::as_str) != Some(self.issuer.as_str()) {
            return Err(Errors::unauthorized("token from another issuer", None));
        }
        match claims.get("exp").and_then(Value::as_i64) {
            Some(exp) if exp + LEEWAY_SECS >= now => {}
            Some(_) => return Err(Errors::unauthorized("token expired", None)),
            None => return Err(Errors::unauthorized("token without expiry", None)),
        }
        if let Some(nbf) = claims.get("nbf").and_then(Value::as_i64) {
            if nbf - LEEWAY_SECS > now {
                return Err(Errors::unauthorized("token not valid yet", None));
            }
        }
        if let Some(audience) = &self.audience {
            let matches = match claims.get("aud") {
                Some(Value::String(aud)) => aud == audience,
                Some(Value::Array(auds)) => auds.iter().any(|a| a.as_str() == Some(audience)),
                _ => false,
            };
            if !matches {
                return Err(Errors::unauthorized("token for another audience", None));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl OauthTokenValidatorTrait for VerifiedTokenValidator {
    async fn validate_token<'a>(&self, token: Option<&'a str>) -> Outcome<UserInfo> {
        let token = token.ok_or_else(|| Errors::unauthorized("missing bearer token", None))?;
        let parts: Vec<&str> = token.split('.').collect();
        let [header_b64, payload_b64, signature_b64] = parts[..] else {
            return Err(Errors::unauthorized("token is not a JWT", None));
        };

        let header: TokenHeader = serde_json::from_slice(&decode_url_safe_no_pad(header_b64)?)
            .map_err(|e| Errors::unauthorized("token header unreadable", Some(Box::new(e))))?;
        // Asymmetric algorithms only: a realm signs with its private key.
        if !matches!(
            header.alg,
            Alg::Rs256 | Alg::Rs384 | Alg::Rs512 | Alg::Ps256 | Alg::Ps384 | Alg::Ps512 | Alg::EdDsa
        ) {
            return Err(Errors::unauthorized("token signed with an unaccepted algorithm", None));
        }

        let key = self.public_key(header.kid.as_deref()).await?;
        let signing_input = &token[..header_b64.len() + 1 + payload_b64.len()];
        let signature = decode_url_safe_no_pad(signature_b64)?;
        key.verify_bytes(signing_input.as_bytes(), &signature, &header.alg)
            .map_err(|e| Errors::unauthorized("invalid token signature", Some(Box::new(e))))?;

        let claims: Value = serde_json::from_slice(&decode_url_safe_no_pad(payload_b64)?)
            .map_err(|e| Errors::unauthorized("token payload unreadable", Some(Box::new(e))))?;
        self.check_claims(&claims)?;

        UserInfo::from_token(token)
            .map_err(|e| Errors::unauthorized("token carries no valid user", Some(Box::new(e))))
    }

    fn user_header(&self) -> &str {
        &self.user_header
    }
}
