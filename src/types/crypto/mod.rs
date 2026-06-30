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

//! Cryptographic data types embedded in VDS / VAR / VC / VP documents.
//!
//! - [`Proof`] — a W3C Data Integrity Proof entry (`type`, `cryptosuite`,
//!   `verificationMethod`, `proofValue`). Serializable to JSON, embedded
//!   in the `proof` array of signed documents.
//! - [`Canon`] — newtype around the JCS-canonicalised string form of a
//!   `serde_json::Value`. The only constructor is `TryFrom<&Value>`, so
//!   having a `Canon` is a compile-time guarantee that the bytes are
//!   canonical. Callers that take `&Canon` cannot accidentally receive
//!   non-canonical data.

mod canon;
mod proof;

pub use canon::Canon;
pub use proof::Proof;
