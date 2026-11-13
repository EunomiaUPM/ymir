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

use std::io::{Read, Write};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use crate::errors::{BadFormat, Errors, Outcome};

/// Minimum length of a status list (VC Bitstring Status List §2.1): 16 KB.
pub const MIN_STATUS_LIST_BITS: usize = 131_072;

/// Bits of a status list; bit 0 is the most significant bit of the first byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitstring(Vec<u8>);

impl Bitstring {
    pub fn new(bits: usize) -> Self {
        Self(vec![0u8; bits.max(MIN_STATUS_LIST_BITS).div_ceil(8)])
    }

    pub fn len(&self) -> usize {
        self.0.len() * 8
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn set(&mut self, index: usize, value: bool) -> Outcome<()> {
        let byte = self.byte_of(index)?;
        let mask = Self::mask(index);
        if value {
            self.0[byte] |= mask;
        } else {
            self.0[byte] &= !mask;
        }
        Ok(())
    }

    pub fn get(&self, index: usize) -> Outcome<bool> {
        let byte = self.byte_of(index)?;
        Ok(self.0[byte] & Self::mask(index) != 0)
    }

    /// GZIP, then multibase base64url without padding (`u` prefix).
    pub fn encode(&self) -> Outcome<String> {
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&self.0)
            .map_err(|e| Errors::parse("Unable to compress the status list", Some(Box::new(e))))?;
        let compressed = gz
            .finish()
            .map_err(|e| Errors::parse("Unable to compress the status list", Some(Box::new(e))))?;
        Ok(format!("u{}", URL_SAFE_NO_PAD.encode(compressed)))
    }

    pub fn decode(encoded: &str) -> Outcome<Self> {
        let b64 = encoded.strip_prefix('u').ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "encodedList is not multibase base64url",
                None,
            )
        })?;
        let compressed = URL_SAFE_NO_PAD.decode(b64).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "encodedList is not valid base64url",
                Some(Box::new(e)),
            )
        })?;
        let mut bytes = Vec::new();
        GzDecoder::new(compressed.as_slice())
            .read_to_end(&mut bytes)
            .map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    "encodedList is not valid GZIP",
                    Some(Box::new(e)),
                )
            })?;
        if bytes.len() * 8 < MIN_STATUS_LIST_BITS {
            return Err(Errors::format(
                BadFormat::Received,
                "Status list is shorter than 131072 bits",
                None,
            ));
        }
        Ok(Self(bytes))
    }

    fn byte_of(&self, index: usize) -> Outcome<usize> {
        match index / 8 {
            byte if byte < self.0.len() => Ok(byte),
            _ => Err(Errors::format(
                BadFormat::Received,
                format!("Status list index {index} is out of range"),
                None,
            )),
        }
    }

    fn mask(index: usize) -> u8 {
        0b1000_0000 >> (index % 8)
    }
}
