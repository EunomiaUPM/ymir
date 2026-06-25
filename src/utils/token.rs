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
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use rand::Rng;

// ===== CRYPTOGRAPHIC TOKEN GENERATION ============================================================

/// Generates a high-entropy, 256-bit opaque security token string.
///
/// Collects randomness via standard local system thread sources, outputting an unpadded
/// network-safe Base64URL serialized layout sequence.
pub fn create_opaque_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

// ===== TEMPORAL EVALUATION ENGINE ================================================================

/// Validates an asset issuance time assertion flag (`iat`) against active host machine clock parameters.
///
/// # Errors
/// Returns an [`Errors::ForbiddenError`] if the token context's declared activation milestone sits
/// inside future temporal horizons.
pub fn is_active(iat: i64) -> Outcome<()> {
    let now = Utc::now().timestamp();
    if now >= iat {
        Ok(())
    } else {
        Err(Errors::forbidden("Token is not yet valid", None))
    }
}

/// Validates an asset absolute lifetime termination barrier flag (`exp`) against host machine clocks.
///
/// # Errors
/// Returns an [`Errors::ForbiddenError`] if active network tracking indicates current milestones
/// have drifted past expiration thresholds.
pub fn has_expired(exp: i64) -> Outcome<()> {
    let now = Utc::now().timestamp();
    if now <= exp {
        Ok(())
    } else {
        Err(Errors::forbidden("Token has expired", None))
    }
}
