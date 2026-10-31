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

use async_trait::async_trait;
use super::{Canon, HasProofPurpose, Proof};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::utils::{OneOrMany, Verifiable, Verified};
use serde::{Deserialize, Serialize};
use crate::capabilities::Verifier;
use crate::types::dids::Did;

/// A document paired with one or more Data Integrity proofs over it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proofed<T> {
    #[serde(flatten)]
    document: T,
    proof: OneOrMany<Proof>,
}

impl<T> Proofed<T> {
    pub fn new(document: T, proof: OneOrMany<Proof>) -> Self {
        Proofed { document, proof }
    }
    pub fn proof(&self) -> &OneOrMany<Proof> {
        &self.proof
    }
}

impl<T> Proofed<T> {
    pub fn unverified_doc(&self) -> &T { &self.document }
}

impl<T> Verified<Proofed<T>> {
    pub fn doc(&self) -> &T { &self.document }
    pub fn signers(&self) -> Vec<&Did> {
        self.proof.iter().map(|p| p.data.verification_method.did()).collect()
    }
}


impl<T: Serialize> Proofed<T> {
    pub fn canon(&self) -> Outcome<Canon> {
        Canon::new(&self.document)
    }
}

#[async_trait]
impl<T> Verifiable for Proofed<T>
where
    T: HasProofPurpose + Serialize + Send + Sync,
{
    type Context = ();

    async fn check(&self, _ctx: &Self::Context) -> Outcome<()> {
        let proofs = self.proof();
        let canon_doc = self.canon()?;

        if proofs.is_empty() {
            return Err(Errors::format(
                BadFormat::Received,
                "Object to verify has no proofs",
                None,
            ));
        }
        for proof in proofs {
            Verifier::verify_proof(&canon_doc, proof, T::PURPOSE).await?;
        }
        Ok(())
    }
}

