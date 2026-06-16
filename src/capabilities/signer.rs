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

use crate::errors::Outcome;
use crate::types::crypto::{Canon, Proof};
use crate::types::jwt::{Jwt, JwtHeader};
use crate::types::keys::{Alg, SigningCtx};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::Value;

pub struct Signer;

impl Signer {
    pub fn sign_embed(sig_ctx: &SigningCtx, canonical: &Canon, alg: Alg) -> Outcome<Proof> {
        let cryptosuite = sig_ctx.key().cryptosuite()?;
        let sig_bytes = sig_ctx.key().sign_bytes(canonical.as_ref(), alg)?;
        let proof_value = format!("z{}", bs58::encode(&sig_bytes).into_string());
        let verification_method = format!("{}#{}", sig_ctx.did().id(), sig_ctx.keys_frag());

        Ok(Proof {
            r#type: "DataIntegrityProof".to_string(),
            cryptosuite,
            verification_method,
            proof_value,
        })
    }

    pub fn sign_enveloped(
        sig_ctx: &SigningCtx,
        typ: &str,
        cty: &str,
        value: &Value,
    ) -> Outcome<Jwt> {
        let kid = format!("{}#{}", sig_ctx.did().id(), sig_ctx.keys_frag());
        let header = JwtHeader {
            alg: sig_ctx.key().alg(),
            typ: Some(typ.to_string()),
            cty: Some(cty.to_string()),
            kid,
            extra: serde_json::Map::new(),
        };

        let header_bytes = serde_json::to_vec(&header)?;
        let payload_bytes = serde_json::to_vec(value)?;

        let header_b64 = URL_SAFE_NO_PAD.encode(&header_bytes);
        let payload_b64 = URL_SAFE_NO_PAD.encode(&payload_bytes);

        let signing_input = format!("{header_b64}.{payload_b64}");
        let sig_bytes = sig_ctx
            .key()
            .sign_bytes(signing_input.as_bytes(), sig_ctx.key().alg())?;
        let sig_b64 = URL_SAFE_NO_PAD.encode(&sig_bytes);

        let jwt = format!("{signing_input}.{sig_b64}");
        Jwt::parse(&jwt)
    }
}
