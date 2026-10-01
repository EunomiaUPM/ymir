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

use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::crypto::{Canon, HasProofPurpose, Proof, ProofOptions, ProofPurpose, Proofed};
use crate::types::dids::kid::Kid;
use crate::types::jwt::{Jwt, JwtHeader};
use crate::types::keys::{Alg, Crv, SigningCtx};
use crate::utils::OneOrMany;
use serde::Serialize;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

/// Centralized Signing Engine managing payload cryptographic proof enrichment.
///
/// Dispatches orchestration routines for executing both embedded W3C Data Integrity Proof
/// assertions and enveloped JSON Web Token (JWT) marshaling signatures over dynamic data contexts.
pub struct Signer;

impl Signer {
    // ===== EMBEDDED DATA INTEGRITY PROOFS ========================================================

    /// Produces a single [`Proof`] over `canon_doc`, hashed per `eddsa-jcs-2022`
    /// (`hash(proofConfig) || hash(doc)`) and signed with `sig_ctx`'s key.
    fn sign_proof(
        sig_ctx: &SigningCtx,
        canon_doc: &Canon,
        proof_purpose: ProofPurpose,
    ) -> Outcome<Proof> {
        let key = sig_ctx.key();

        if key.alg() != Alg::EdDsa {
            return Err(Errors::not_impl("Only EdDSA signing is supported", None));
        }

        if key.crv() != Some(Crv::Ed25519) {
            return Err(Errors::not_impl("Only Ed25519 signing is supported", None));
        }

        let cryptosuite = key.cryptosuite()?;
        let verification_method = Kid::new(sig_ctx.did().clone(), sig_ctx.keys_frag());

        let options = ProofOptions {
            context: canon_doc.context().cloned(),
            r#type: "DataIntegrityProof".to_string(),
            cryptosuite,
            proof_purpose,
            verification_method,
        };

        let hash_data = options.hash_data(canon_doc)?;

        let sig = sig_ctx.key().sign_bytes(&hash_data, key.alg())?;

        Ok(Proof {
            data: options,
            proof_value: format!("z{}", bs58::encode(&sig).into_string()),
        })
    }

    /// Secures a document with a single proof, reading the proof purpose from
    /// the document type itself.
    pub fn sign_embedded<T>(document: T, sig_ctx: &SigningCtx) -> Outcome<Proofed<T>>
    where
        T: Serialize + HasProofPurpose,
    {
        let canon = Canon::new(&document)?;
        let proof = Self::sign_proof(sig_ctx, &canon, T::PURPOSE)?;

        Ok(Proofed::new(document, OneOrMany::One(proof)))
    }

    /// Secures a document with the proof of every signing context, reading the
    /// proof purpose from the document type itself.
    pub fn sign_embedded_set<T>(document: T, sigs_ctx: &[SigningCtx]) -> Outcome<Proofed<T>>
    where
        T: Serialize + HasProofPurpose,
    {
        if sigs_ctx.is_empty() {
            return Err(Errors::format(
                BadFormat::Sent,
                "Empty signature context",
                None,
            ));
        }

        let canon = Canon::new(&document)?;
        let proof = sigs_ctx
            .iter()
            .map(|ctx| Self::sign_proof(ctx, &canon, T::PURPOSE))
            .collect::<Outcome<Vec<Proof>>>()?;

        Ok(Proofed::new(document, OneOrMany::Many(proof)))
    }

    // ===== ENVELOPED JSON WEB TOKENS =============================================================

    /// Encapsulates dynamic structured JSON data inside an authoritative compact cryptographic [`Jwt`] envelope.
    pub fn sign_enveloped<T>(sig_ctx: &SigningCtx, typ: &str, cty: &str, value: &T) -> Outcome<Jwt>
    where
        T: Serialize + ?Sized,
    {
        let kid = Kid::new(sig_ctx.did().clone(), sig_ctx.keys_frag());
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
