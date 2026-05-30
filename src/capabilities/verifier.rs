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

use serde::de::DeserializeOwned;
use super::{Kid};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::crypto::{Canon, Proof};
use crate::types::jwt::Jwt;
use serde_json::Value;
use crate::types::keys::Alg;

pub struct Verifier;

impl Verifier {
    pub async fn verify_embed(value: &Value) -> Outcome<()> {
        let mut value = value.clone();
        let proof_value = value
            .as_object_mut()
            .and_then(|obj| obj.remove("proof"))
            .ok_or_else(|| Errors::format(BadFormat::Received, "Missing proof", None))?;

        let proofs: Vec<Proof> = serde_json::from_value(proof_value)?;

        let canonical = Canon::try_from(&value)?;
        for proof in proofs {
            Self::verify_single_proof(&canonical, &proof).await?;
        }
        Ok(())
    }
    async fn verify_single_proof(value: &Canon, proof: &Proof) -> Outcome<()> {
        let kid = Kid::parse(&proof.verification_method)?;
        let alg = Alg::from_cryptosuite(&proof.cryptosuite);
        let key = kid.get_key(&alg).await?;

        let b58 = proof.proof_value.strip_prefix('z').ok_or_else(|| {
            Errors::parse("proofValue must start with 'z' (multibase base58btc)", None)
        })?;
        let sig = bs58::decode(b58)
            .into_vec()
            .map_err(|e| Errors::parse("base58 decode of proofValue failed", Some(Box::new(e))))?;

        key.verify_bytes(value.as_ref(), &sig)
    }

    pub async fn verify_enveloped<T: DeserializeOwned>(
        jwt: &Jwt,
        expected_aud: Option<&str>,
    ) -> Outcome<(Kid, T)> {
        let kid = Kid::parse(&jwt.header().kid)?;
        let key = kid.get_key(&jwt.header().alg).await?;

        key.verify_bytes(jwt.signing_input(), jwt.signature())?;

        let value_payload: Value = jwt.unsafe_claims()?;
        if let Some(expected) = expected_aud {
            let matches = match &value_payload["aud"] {
                Value::String(s) => s == expected,
                Value::Array(arr) => arr.iter().any(|v| v.as_str() == Some(expected)),
                _ => false,
            };
            if !matches {
                return Err(Errors::format(
                    BadFormat::Received,
                    format!("audience mismatch: expected '{expected}'"),
                    None,
                ));
            }
        }
        let payload: T = serde_json::from_value(value_payload)?;
        Ok((kid, payload))
    }
}
