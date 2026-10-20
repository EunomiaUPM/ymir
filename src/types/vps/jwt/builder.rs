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

use std::marker::PhantomData;

use chrono::{DateTime, Utc};

use super::{VpPayload, VpPayloadV1};
use crate::types::jwt::JwtClaims;
use crate::types::vcs::W3cDataModelVersion;
use crate::types::vps::{PresentedCredential, VpDocument};
use crate::utils::{Missing, OneOrMany, Present};

/// Builds a VP's JWT claims; it does not compile without the presentation (`VP`), `aud` (`AUD`)
/// and `nonce` (`NONCE`). `iss`, `sub` and `jti` come from the presentation.
pub struct VpJwtClaimsBuilder<VP, AUD, NONCE> {
    w3c_data_model_version: W3cDataModelVersion,
    claims: JwtClaims<()>,
    vp: Option<VpDocument>,
    _marker: PhantomData<(VP, AUD, NONCE)>,
}

impl VpJwtClaimsBuilder<Missing, Missing, Missing> {
    pub fn new(w3c_data_model_version: W3cDataModelVersion) -> Self {
        VpJwtClaimsBuilder {
            w3c_data_model_version,
            claims: JwtClaims::default(),
            vp: None,
            _marker: PhantomData,
        }
    }
}

impl<VP, AUD, NONCE> VpJwtClaimsBuilder<VP, AUD, NONCE> {
    /// The presentation of `vc_jwts` by `holder`; `id` is also the JWT's `jti`.
    pub fn presentation(
        mut self,
        id: impl Into<String>,
        holder: impl Into<String>,
        vc_jwts: Vec<String>,
    ) -> VpJwtClaimsBuilder<Present, AUD, NONCE> {
        let version = self.w3c_data_model_version;
        let (id, holder) = (id.into(), holder.into());
        self.claims = self.claims.with_iss(&holder).with_sub(&holder).with_jti(&id);
        self.vp = Some(VpDocument {
            context: vec![version.context().to_string()],
            id,
            r#type: OneOrMany::Many(vec!["VerifiablePresentation".to_string()]),
            holder: Some(holder),
            verifiable_credential: OneOrMany::Many(
                vc_jwts
                    .into_iter()
                    .map(|jwt| PresentedCredential::new(jwt, version))
                    .collect(),
            ),
        });
        self.state()
    }

    pub fn aud(mut self, aud: impl Into<String>) -> VpJwtClaimsBuilder<VP, Present, NONCE> {
        self.claims = self.claims.with_aud(aud);
        self.state()
    }

    pub fn nonce(mut self, nonce: impl Into<String>) -> VpJwtClaimsBuilder<VP, AUD, Present> {
        self.claims = self.claims.with_nonce(nonce);
        self.state()
    }

    pub fn iat(mut self, iat: DateTime<Utc>) -> Self {
        self.claims = self.claims.with_iat(iat);
        self
    }

    pub fn exp(mut self, exp: DateTime<Utc>) -> Self {
        self.claims = self.claims.with_exp(exp);
        self
    }

    fn state<V, A, N>(self) -> VpJwtClaimsBuilder<V, A, N> {
        VpJwtClaimsBuilder {
            w3c_data_model_version: self.w3c_data_model_version,
            claims: self.claims,
            vp: self.vp,
            _marker: PhantomData,
        }
    }
}

impl VpJwtClaimsBuilder<Present, Present, Present> {
    /// `nbf` only in VC DM 1.1, as `iat`; VC-JOSE-COSE does not recommend it.
    pub fn build(self) -> JwtClaims<VpPayload> {
        let vp = self.vp.unwrap();
        match self.w3c_data_model_version {
            W3cDataModelVersion::V1 => {
                let iat = self.claims.iat().and_then(|t| DateTime::from_timestamp(t, 0));
                let claims = match iat {
                    Some(iat) => self.claims.with_nbf(iat),
                    None => self.claims,
                };
                claims.with_body(VpPayload::V1(VpPayloadV1 { vp }))
            }
            W3cDataModelVersion::V2 => self.claims.with_body(VpPayload::V2(vp)),
        }
    }
}
