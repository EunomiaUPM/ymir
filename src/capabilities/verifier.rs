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

use super::Did;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::crypto::{Canon, Proof};
use serde_json::Value;
use crate::types::jwt::Jwt;

pub struct Verifier;

impl Verifier {
    /// Verify all Data Integrity proofs embedded in a document.
    ///
    /// The procedure for each proof entry:
    ///
    /// 1. Resolve the DID in `verificationMethod` to a public key.
    /// 2. Reject if the declared `cryptosuite` is not supported.
    /// 3. Decode the multibase z-base58btc `proofValue` to raw bytes.
    /// 4. Verify the signature against the JCS-canonical bytes of the
    ///    document **with the `proof` field removed**.
    ///
    /// # Errors
    ///
    /// Returns `Err` on the **first** proof that fails any of the steps.
    /// All-or-nothing semantics; not quorum-aware. A quorum-aware variant
    /// is left as future work (see paper Section V).
    ///
    /// # Async
    ///
    /// `async` because DID resolution may require HTTP (`did:web`). For
    /// pure `did:jwk` flows, resolution is local and await is a
    /// no-op.
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
        let did = Did::parse(&proof.verification_method)?;
        let key = did.get_key().await?;
        match proof.cryptosuite.as_str() {
            "eddsa-jcs-2022" => {}
            cryptosuite => {
                return Err(Errors::not_impl(
                    format!("Cryptosuite {cryptosuite} not implemented"),
                    None,
                ));
            }
        }

        // proofValue is multibase z-base58btc (W3C VC Data Integrity).
        let b58 = proof.proof_value.strip_prefix('z').ok_or_else(|| {
            Errors::parse(
                "proofValue must start with 'z' (multibase base58btc)",
                None,
            )
        })?;
        let sig = bs58::decode(b58)
            .into_vec()
            .map_err(|e| Errors::parse("base58 decode of proofValue failed", Some(Box::new(e))))?;

        key.verify_bytes(value.as_ref(), &sig)
    }

    /// Verify a JWS compact serialization produced by
    /// [`Signer::sign_enveloped`].
    ///
    /// The procedure:
    ///
    /// 1. Split the JWS into header / payload / signature segments.
    /// 2. Read `kid` from the header and resolve it to a public key.
    /// 3. Base64url-decode the signature segment.
    /// 4. Verify against the signing input `<header>.<payload>` bytes.
    ///
    /// # Key resolution
    ///
    /// The verifier uses the `kid` claim from the JWS header (RFC 7515)
    /// rather than the `iss` claim from the JWT payload. This matches
    /// what [`Signer::sign_enveloped`] writes — `kid` is the DID URL of
    /// the signing key.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the signature is valid. Does not return the parsed
    /// payload; the caller can decode it from the second segment if
    /// needed.
    pub async fn verify_enveloped(jwt: &Jwt, expected_aud: Option<&str>) -> Outcome<()> {
        let kid = jwt.expect_kid()?;

        let did = Did::parse(kid)?;
        let key = did.get_key().await?;

        key.verify_bytes(jwt.signing_input(), jwt.signature())?;

        if let Some(expected) = expected_aud {
            let payload: Value = jwt.claims()?;
            let matches = match &payload["aud"] {
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

        Ok(())
    }
}
