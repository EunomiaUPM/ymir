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

use super::JwtHeader;
use crate::errors::{BadFormat, Errors, Outcome};
use crate::utils::{decode_url_safe_no_pad, get_claim, get_opt_claim};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct Jwt {
    raw: String,
    header: JwtHeader,
    payload: Value,
    signature: Vec<u8>,
    signing_input_len: usize,
}
impl Jwt {
    pub fn parse(jwt: impl Into<String>) -> Outcome<Self> {
        let raw = jwt.into();
        let parts: Vec<&str> = raw.split('.').collect();
        if parts.len() != 3 {
            return Err(Errors::format(
                BadFormat::Received,
                "JWT has wrong format",
                None,
            ));
        }

        let header_bytes = decode_url_safe_no_pad(parts[0])?;
        let payload_bytes = decode_url_safe_no_pad(parts[1])?;
        let signature = decode_url_safe_no_pad(parts[2])?;

        let header: JwtHeader = serde_json::from_slice(&header_bytes)?;
        let payload: Value = serde_json::from_slice(&payload_bytes)?;
        let signing_input_len = parts[0].len() + 1 + parts[1].len();

        Ok(Self {
            raw,
            header,
            payload,
            signature,
            signing_input_len,
        })
    }

    pub fn header(&self) -> &JwtHeader {
        &self.header
    }
    pub fn payload(&self) -> &Value {
        &self.payload
    }
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
    pub fn signing_input(&self) -> &[u8] {
        self.raw[..self.signing_input_len].as_bytes()
    }
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn claims<T: DeserializeOwned>(&self) -> Outcome<T> {
        serde_json::from_value(self.payload.clone())
            .map_err(|e| Errors::parse("claims shape mismatch", Some(Box::new(e))))
    }
    pub fn expect_kid(&self) -> Outcome<&str> {
        self.header
            .kid
            .as_deref()
            .ok_or_else(|| Errors::format(BadFormat::Received, "kid missing in JWS header", None))
    }

    pub fn claim(&self, path: &[&str]) -> Outcome<String> {
        get_claim(self.payload(), path)
    }

    pub fn claim_opt(&self, path: &[&str]) -> Outcome<Option<String>> {
        get_opt_claim(self.payload(), path)
    }
}

impl std::fmt::Display for Jwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw)
    }
}

impl AsRef<str> for Jwt {
    fn as_ref(&self) -> &str {
        &self.raw
    }
}
