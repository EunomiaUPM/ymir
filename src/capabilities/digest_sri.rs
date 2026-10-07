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

use crate::types::crypto::{Canon, HashAlg};
use sha2::{Digest, Sha256, Sha384, Sha512};

/// Integrity hashing utility for canonical data buffers.
///
/// Computes a raw digest over a canonicalized structural envelope ([`Canon`]);
/// callers format it (SRI's `sha256-<base64>`, a content identifier's
/// `sha256:<hex>`, ...) for their own purpose.
pub struct DigestSRI;

impl DigestSRI {
    pub fn digest(canonical: &Canon, hash_alg: HashAlg) -> Vec<u8> {
        match hash_alg {
            HashAlg::Sha256 => Sha256::digest(canonical.as_ref()).to_vec(),
            HashAlg::Sha384 => Sha384::digest(canonical.as_ref()).to_vec(),
            HashAlg::Sha512 => Sha512::digest(canonical.as_ref()).to_vec(),
        }
    }
}
