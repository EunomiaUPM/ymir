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

use crate::errors::{Errors, Outcome};
use crate::types::crypto::Canon;
use base64::{Engine, engine::general_purpose};
use sha2::{Digest, Sha256};

/// Subresource Integrity (SRI) hashing utility for canonical data buffers.
///
/// Provides deterministic cryptographic checksum generation and verification
/// over canonicalized structural envelopes ([`Canon`]) using standard web-integrity layouts.
pub struct DigestSRI;

impl DigestSRI {
    // ===== DIGEST GENERATION =====================================================================

    /// Computes the SHA-256 integrity hash over a canonical representation and encodes it as an SRI token.
    ///
    /// Returns a standardized formatted string: `sha256-<base64_payload>`.
    pub fn digest(canonical: &Canon) -> String {
        let hash = Sha256::digest(canonical.as_ref());
        let b64 = general_purpose::STANDARD.encode(hash);
        format!("sha256-{}", b64)
    }

    // ===== VALIDATION PIPELINE ===================================================================

    /// Assesses an inbound SRI metadata string pattern against a locally computed token.
    ///
    /// # Errors
    /// Returns an [`Errors::FeatureNotImplError`] if the provided SRI string does not match the
    /// required `sha256-` prefix taxonomy.
    pub fn validate_json_sri(canonical: &Canon, sri: impl Into<String>) -> Outcome<bool> {
        let sri = sri.into();

        if !sri.starts_with("sha256-") {
            return Err(Errors::not_impl(
                "Digest SRI only accepts sha 256 right now",
                None,
            ));
        }

        let computed = Self::digest(canonical);
        Ok(computed == sri)
    }
}
