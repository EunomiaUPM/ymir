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

use super::Kid;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::crypto::{Canon, Proof};
use crate::types::jwt::Jwt;
use crate::types::keys::Alg;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Centralized Cryptographic Verification Engine validating asset authenticity.
///
/// Processes incoming data boundaries by resolving internal key material anchors
/// and evaluating structural correctness of embedded proofs or enveloped network tokens.
pub struct Verifier;

impl Verifier {
    // ===== EMBEDDED PROOF VALIDATION =============================================================

    /// Evaluates structural embedded signature suites appended under the standard JSON `"proof"` property boundary.
    ///
    /// # Errors
    /// Returns an [`Errors::FormatError`] if the data object lacks valid structural proofs or
    /// if any single evaluated data cryptographic signature step encounters mathematical verification mismatches.
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

    /// Isolated logical runner verifying an individual extracted W3C structural [`Proof`] instance.
    async fn verify_single_proof(value: &Canon, proof: &Proof) -> Outcome<()> {
        let kid = Kid::parse(&proof.verification_method)?;
        let alg = Alg::from_cryptosuite(&proof.cryptosuite);
        let key = kid.get_key().await?;

        let b58 = proof.proof_value.strip_prefix('z').ok_or_else(|| {
            Errors::parse("proofValue must start with 'z' (multibase base58btc)", None)
        })?;
        let sig = bs58::decode(b58)
            .into_vec()
            .map_err(|e| Errors::parse("base58 decode of proofValue failed", Some(Box::new(e))))?;

        key.verify_bytes(value.as_ref(), &sig, &alg)
    }

    // ===== ENVELOPED JWT VALIDATION ==============================================================

    /// Unwraps and verifies an authoritative compact network [`Jwt`], validating cryptographic bounds and audiences.
    ///
    /// Automatically performs dynamic deserialization into the requested payload model structure target `T`.
    ///
    /// # Errors
    /// Returns an [`Errors::FormatError`] if verification bounds break or if the token's structural
    /// target `"aud"` vector claims fail to match the expected parameter constraint layout.
    pub async fn verify_enveloped<T: DeserializeOwned>(
        jwt: &Jwt,
        expected_aud: Option<&str>,
    ) -> Outcome<(Kid, T)> {
        let kid = Kid::parse(&jwt.header().kid)?;
        let key = kid.get_key().await?;
        key.verify_bytes(jwt.signing_input(), jwt.signature(), &jwt.header().alg)?;

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