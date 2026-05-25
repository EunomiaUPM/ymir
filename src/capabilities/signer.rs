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
use crate::types::jwt::Jwt;
use crate::types::wallet::fafnir::SigningCtx;
use crate::utils::HasId;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::Value;

pub struct Signer;

impl Signer {
    pub fn sign_embed(canonical: &Canon, ctx: &SigningCtx) -> Outcome<Proof> {
        let cryptosuite = ctx.key().cryptosuite()?;
        let sig_bytes = &ctx.key().sign_bytes(canonical.as_ref())?;
        let proof_value = format!("z{}", bs58::encode(&sig_bytes).into_string());

        Ok(Proof {
            r#type: "DataIntegrityProof".to_string(),
            cryptosuite: cryptosuite.to_string(),
            verification_method: ctx.did().id().to_string(),
            proof_value,
        })
    }

    pub fn sign_enveloped(typ: &str, cty: &str, value: &Value, ctx: &SigningCtx) -> Outcome<Jwt> {
        let alg = &ctx.key().jws_alg();

        let header = serde_json::json!({
            "alg": alg,
            // `kid` apunta al verificationMethod concreto dentro del DID
            // document (formato `<did>#<key-id>`). El verifier hace
            // `Did::parse_from_kid` y luego busca el VM por ese mismo
            // id en el doc resuelto.
            "kid": format!("{}#{}", ctx.did().id(), ctx.key().id()),
            "typ": typ,
            "cty": cty,
        });

        let header_bytes = serde_json::to_vec(&header)?;
        let payload_bytes = serde_json::to_vec(value)?;

        let header_b64 = URL_SAFE_NO_PAD.encode(&header_bytes);
        let payload_b64 = URL_SAFE_NO_PAD.encode(&payload_bytes);

        let signing_input = format!("{header_b64}.{payload_b64}");
        let sig_bytes = ctx.key().sign_bytes(signing_input.as_bytes())?;
        let sig_b64 = URL_SAFE_NO_PAD.encode(&sig_bytes);

        let jwt = format!("{signing_input}.{sig_b64}");
        Jwt::parse(jwt)
    }
}
