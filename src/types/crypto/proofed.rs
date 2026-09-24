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

use super::{Canon, Proof};
use crate::errors::Outcome;
use crate::utils::OneOrMany;
use serde::{Deserialize, Serialize};

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
    pub fn doc(&self) -> &T {
        &self.document
    }
    pub fn proof(&self) -> &OneOrMany<Proof> {
        &self.proof
    }
}

impl<T> Proofed<T>
where
    T: Serialize,
{
    pub fn canon(&self) -> Outcome<Canon> {
        Canon::new(self.doc())
    }
}
