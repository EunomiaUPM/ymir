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
use crate::types::crypto::Canon;
use crate::types::keys::RetrievedKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proof {
    pub r#type: String,
    pub cryptosuite: String,
    #[serde(rename = "verificationMethod")]
    pub verification_method: String,
    #[serde(rename = "proofValue")]
    pub proof_value: String,
}

impl Proof {
    /// Decode the multibase z-base58btc proofValue into raw signature bytes.
    pub fn signature(&self) -> Outcome<Vec<u8>> {
        let b58 = self.proof_value.strip_prefix('z').ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "proofValue must start with 'z' (multibase base58btc)",
                None,
            )
        })?;
        bs58::decode(b58)
            .into_vec()
            .map_err(|e| Errors::parse("base58 decode of proofValue failed", Some(Box::new(e))))
    }

    /// Ensure the declared cryptosuite is one we support.
    pub fn ensure_supported_cryptosuite(&self) -> Outcome<()> {
        match self.cryptosuite.as_str() {
            "eddsa-jcs-2022" => Ok(()),
            other => Err(Errors::not_impl(
                format!("Cryptosuite {other} not implemented"),
                None,
            )),
        }
    }

    /// Verify this proof against canonical bytes with an already-resolved key.
    /// Pure: no DID resolution, no IO. Cryptosuite check + decode + verify.
    pub fn verify_with(&self, key: &RetrievedKey, canon: &Canon) -> Outcome<()> {
        self.ensure_supported_cryptosuite()?;
        let sig = self.signature()?;
        key.verify_bytes(canon.as_ref(), &sig)
    }
}
