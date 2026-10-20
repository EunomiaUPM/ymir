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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::crypto::{HasProofPurpose, ProofPurpose};
use crate::types::jwt::{HasJwtType, JwtType};
use crate::utils::OneOrMany;

/// A JWT payload: the registered claims (RFC 7519 §4.1, plus the IANA-registered `nonce`) and
/// what is specific to each kind of JWT, `body`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JwtClaims<T> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    iss: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sub: Option<String>,
    /// A string or an array of them (RFC 7519 §4.1.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    aud: Option<OneOrMany<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    jti: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nbf: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exp: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    iat: Option<i64>,
    #[serde(flatten)]
    body: T,
}

impl<T> JwtClaims<T> {
    /// Claims around `body`, all registered claims unset; set them with the chained setters.
    pub fn new(body: T) -> Self {
        JwtClaims {
            iss: None,
            sub: None,
            aud: None,
            nonce: None,
            jti: None,
            nbf: None,
            exp: None,
            iat: None,
            body,
        }
    }

    pub fn iss(&self) -> Option<&str> {
        self.iss.as_deref()
    }

    pub fn sub(&self) -> Option<&str> {
        self.sub.as_deref()
    }

    pub fn aud(&self) -> Option<&OneOrMany<String>> {
        self.aud.as_ref()
    }

    pub fn nonce(&self) -> Option<&str> {
        self.nonce.as_deref()
    }

    pub fn jti(&self) -> Option<&str> {
        self.jti.as_deref()
    }

    pub fn nbf(&self) -> Option<i64> {
        self.nbf
    }

    pub fn exp(&self) -> Option<i64> {
        self.exp
    }

    pub fn iat(&self) -> Option<i64> {
        self.iat
    }

    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn with_iss(mut self, iss: impl Into<String>) -> Self {
        self.iss = Some(iss.into());
        self
    }

    pub fn with_sub(mut self, sub: impl Into<String>) -> Self {  
        self.sub = Some(sub.into());
        self
    }

    /// A single audience; for several, set `aud` directly.
    pub fn with_aud(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(OneOrMany::One(aud.into()));
        self
    }

    pub fn with_nonce(mut self, nonce: impl Into<String>) -> Self {
        self.nonce = Some(nonce.into());
        self
    }

    pub fn with_jti(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    pub fn with_nbf(mut self, nbf: DateTime<Utc>) -> Self {
        self.nbf = Some(nbf.timestamp());
        self
    }

    pub fn with_exp(mut self, exp: DateTime<Utc>) -> Self {
        self.exp = Some(exp.timestamp());
        self
    }

    pub fn with_iat(mut self, iat: DateTime<Utc>) -> Self {
        self.iat = Some(iat.timestamp());
        self
    }

    /// The same claims around another body; builders start from `JwtClaims<()>`.
    pub fn with_body<U>(self, body: U) -> JwtClaims<U> {
        JwtClaims {
            iss: self.iss,
            sub: self.sub,
            aud: self.aud,
            nonce: self.nonce,
            jti: self.jti,
            nbf: self.nbf,
            exp: self.exp,
            iat: self.iat,
            body,
        }
    }


    /// Whether `aud` names `audience`.
    pub fn has_audience(&self, audience: &str) -> bool {
        self.aud
            .as_ref()
            .is_some_and(|aud| aud.iter().any(|a| a == audience))
    }
}

/// A JWT is signed for what its body asks.
impl<T: HasProofPurpose> HasProofPurpose for JwtClaims<T> {
    const PURPOSE: ProofPurpose = T::PURPOSE;
}

/// A JWT's `typ` and `cty` are those of its body.
impl<T: HasJwtType> HasJwtType for JwtClaims<T> {
    const TYP: JwtType = T::TYP;
    const CTY: Option<&'static str> = T::CTY;

    fn jwt_type(&self) -> JwtType {
        self.body.jwt_type()
    }

    fn content_type(&self) -> Option<&'static str> {
        self.body.content_type()
    }

    fn accepts_typ(typ: Option<&JwtType>) -> bool {
        T::accepts_typ(typ)
    }
}
