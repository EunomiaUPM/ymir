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

//! Cryptographic data types for signed and canonicalised documents.
//!
//! - [`Proof`] / [`ProofOptions`] — a W3C Data Integrity Proof entry
//!   (`type`, `cryptosuite`, `proofPurpose`, `verificationMethod`,
//!   `proofValue`), split so the signed fields (`ProofOptions`) are
//!   distinct from the signature itself.
//! - [`Canon`] — newtype around the JCS-canonicalised string form of a
//!   serializable value. The only constructor is [`Canon::new`], so having
//!   a `Canon` is a compile-time guarantee that the bytes are canonical.
//! - [`HashAlg`] — the hash algorithms usable for content digests.
//! - [`ProofPurpose`] / [`HasProofPurpose`] — the purpose a proof declares,
//!   and the purpose a given document type requires.
//! - [`Proofed`] — a document paired with its proof(s).

mod canon;
mod hash_alg;
mod proof;
mod proofed;
mod purpose;

pub use canon::Canon;
pub use hash_alg::HashAlg;
pub use proof::{Proof, ProofOptions};
pub use proofed::Proofed;
pub use purpose::{HasProofPurpose, ProofPurpose};
