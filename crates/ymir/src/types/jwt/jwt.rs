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

use std::fmt;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

use super::{HasJwtType, JwtClaims, JwtHeader, JwtHeaderTrait, JwtType};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::crypto::HasProofPurpose;
use crate::types::dids::Did;
use crate::utils::{decode_url_safe_no_pad, Verifiable, Verified};

/// A compact JWS whose claims read as `JwtClaims<T>`; `T = Value` when the shape is not known.
/// Its claims are trusted only once [`Verified`].
#[derive(Debug, Clone)]
pub struct Jwt<T> {
    raw: String,
    header: JwtHeader,
    claims: JwtClaims<T>,
    signature: Vec<u8>,
    signing_input_len: usize,
}

impl<T: DeserializeOwned> Jwt<T> {
    pub fn parse(jwt: &str) -> Outcome<Self> {
        let raw = jwt.to_string();
        let parts: Vec<&str> = raw.split('.').collect();
        if parts.len() != 3 {
            return Err(Errors::format(
                BadFormat::Received,
                "JWT has wrong format",
                None,
            ));
        }

        let header_bytes = decode_url_safe_no_pad(parts[0])?;
        let payload_bytes = decode_url_safe_no_pad(parts[1])?;
        let signature = decode_url_safe_no_pad(parts[2])?;

        let header: JwtHeader = serde_json::from_slice(&header_bytes)?;
        header.check_single_key()?;
        let claims: JwtClaims<T> = serde_json::from_slice(&payload_bytes)
            .map_err(|e| Errors::parse("JWT claims do not have the expected shape", Some(Box::new(e))))?;
        let signing_input_len = parts[0].len() + 1 + parts[1].len();

        Ok(Self {
            raw,
            header,
            claims,
            signature,
            signing_input_len,
        })
    }
}

impl<T> JwtHeaderTrait for Jwt<T> {
    fn header(&self) -> &JwtHeader {
        &self.header
    }
}

impl<T: HasJwtType> HasJwtType for Jwt<T> {
    const TYP: JwtType = T::TYP;
    const CTY: Option<&'static str> = T::CTY;

    fn jwt_type(&self) -> JwtType { self.claims.jwt_type() }
    fn content_type(&self) -> Option<&'static str> { self.claims.content_type() }
    fn accepts_typ(typ: Option<&JwtType>) -> bool { T::accepts_typ(typ) }
}

impl<T> Jwt<T> {
    /// The claims before checking the signature: not to be trusted.
    pub fn unverified_claims(&self) -> &JwtClaims<T> {
        &self.claims
    }
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
    pub fn signing_input(&self) -> &[u8] {
        self.raw[..self.signing_input_len].as_bytes()
    }
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}


impl<T> Verified<Jwt<T>> {
    pub fn claims(&self) -> &JwtClaims<T> {
        &self.claims
    }
    pub fn body(&self) -> &T {
        &self.claims.body()
    }
    /// Who signed it, from its header (`kid`, or the `did:jwk` of a `jwk`).
    pub fn signer(&self) -> Outcome<&Did> {
        Ok(self.kid()?.did())
    }
}

#[async_trait]
impl<T> Verifiable for Jwt<T>
where
    T: HasProofPurpose + HasJwtType + Send + Sync,
{
    type Context = Option<String>;

    async fn check(&self, expected_aud: &Self::Context) -> Outcome<()> {
        let header = self.header();
        if !T::accepts_typ(header.typ.as_ref()) {
            let got = header.typ.as_ref().map(ToString::to_string).unwrap_or_default();
            return Err(Errors::security(
                format!("JWT typ '{got}' is not the expected '{}'", T::TYP),
                None,
            ));
        }

        let kid = header.key().kid()?;
        let did_doc = kid.did().resolve().await?;
        let key = did_doc.resolve_key(kid, &T::PURPOSE)?;

        key.verify_bytes(self.signing_input(), self.signature(), self.header().alg())?;

        if let Some(expected) = expected_aud {
            if !self.claims.has_audience(expected) {
                return Err(Errors::unauthorized(format!("audience mismatch: expected '{expected}'"), None));
            }
        }
        Ok(())
    }
}

impl<T> fmt::Display for Jwt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

impl<T> AsRef<str> for Jwt<T> {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}
