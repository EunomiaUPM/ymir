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

use std::sync::Arc;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose, Engine};
use serde_json::Value;
use crate::errors::Outcome;
use crate::services::client::ClientTrait;
use crate::utils::ResponseExt;

pub struct DigestSRI;

impl DigestSRI {
    pub fn digest(value: &Value) -> Outcome<String> {
        let canonical = json_canon::to_string(value)?;
        let hash = Sha256::digest(canonical.as_bytes());
        let b64 = general_purpose::STANDARD.encode(hash);
        Ok(format!("sha256-{}", b64))
    }

    pub fn validate_json_sri(value: &Value, sri: impl Into<String>) -> Outcome<bool> {
        let sri = sri.into();
        let computed = Self::digest(value)?;
        Ok(computed == sri)
    }

    pub async fn validate_http_sri(sri: impl Into<String>, url: &str, client: Arc<dyn ClientTrait>) -> Outcome<bool> {
        let sri = sri.into();
        let res = client.get(url, None).await?;

        if !res.status().is_success() {
            return Ok(false);
        }

        let body: Value = res.parse_json().await?;
        Self::validate_json_sri(&body, sri)
    }
}