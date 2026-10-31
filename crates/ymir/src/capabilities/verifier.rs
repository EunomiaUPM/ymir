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

use crate::errors::{Errors, Outcome};
use crate::types::crypto::{Canon, HasProofPurpose, Proof, ProofPurpose, Proofed};
use crate::types::jwt::{HasJwtType, Jwt};
use crate::types::keys::{Alg, Cryptosuite};
use serde::Serialize;
use crate::utils::Verified;

/// Centralized Cryptographic Verification Engine validating asset authenticity.
///
/// Processes incoming data boundaries by resolving internal key material anchors
/// and evaluating structural correctness of embedded proofs or enveloped network tokens.
pub struct Verifier;

impl Verifier {
    // ===== EMBEDDED PROOF VALIDATION =============================================================

    /// Evaluates every proof attached to `value`, checking each against the
    /// proof purpose that `T` requires.
    ///
    /// # Errors
    /// Returns an [`Errors::FormatError`] if the document carries no proofs, or an
    /// [`Errors::SecurityError`] if any proof fails to verify against `T::PURPOSE`.
    pub async fn verify_embedded<T>(value: Proofed<T>) -> Outcome<Verified<Proofed<T>>>
    where
        T: Serialize + HasProofPurpose + Send + Sync,
    {
        Verified::verify(value, &()).await
    }

    /// Isolated logical runner verifying an individual extracted W3C structural [`Proof`] instance,
    /// resolving its verification method and checking that the document's DID authorises it
    /// for `proof_purpose` (CID v1.0's verification relationship requirement).
    pub(crate) async fn verify_proof(
        canon_doc: &Canon,
        proof: &Proof,
        proof_purpose: ProofPurpose,
    ) -> Outcome<()> {
        if proof.data.r#type != "DataIntegrityProof" || proof.data.cryptosuite != Cryptosuite::EddsaJcs2022 {
            return Err(Errors::security("unexpected proof type or cryptosuite", None));
        }

        if let Some(ctx) = &proof.data.context {
            if canon_doc.context() != Some(ctx) {
                return Err(Errors::security(
                    "proof @context does not match document",
                    None,
                ));
            }
        }

        if proof.data.proof_purpose != proof_purpose {
            return Err(Errors::security(
                format!(
                    "Unexpected proofPurpose: got {}, expected {}",
                    proof.data.proof_purpose, proof_purpose
                ),
                None,
            ));
        }

        let alg = Alg::from_cryptosuite(&proof.data.cryptosuite);
        let kid = &proof.data.verification_method;
        let did_doc = kid.did().resolve().await?;
        let key = did_doc.resolve_key(kid, &proof_purpose)?;

        let sig = proof.signature()?;

        let hash_data = proof.data.hash_data(canon_doc)?;

        key.verify_bytes(&hash_data, &sig, &alg)
    }

    // ===== ENVELOPED JWT VALIDATION ==============================================================

    /// Unwraps and verifies an authoritative compact network [`Jwt`], validating cryptographic bounds and audiences.
    ///
    /// Automatically performs dynamic deserialization into the requested payload model structure target `T`.
    /// Resolves the signing key through the issuer's DID Document, requiring it be authorised for `T::PURPOSE`.
    /// Rejects a `typ` that `T` does not accept ([`HasJwtType`]).
    ///
    /// # Errors
    /// Returns an [`Errors::FormatError`] if verification bounds break or if the token's structural
    /// target `"aud"` vector claims fail to match the expected parameter constraint layout.
    pub async fn verify_enveloped<T>(jwt: Jwt<T>, expected_aud: Option<String>) -> Outcome<Verified<Jwt<T>>>
    where
        T: HasProofPurpose + HasJwtType + Send + Sync,
    {
        Verified::verify(jwt, &expected_aud).await
    }
}
