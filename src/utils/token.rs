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

use crate::errors::{BadFormat, Errors, Outcome};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde_json::Value;

pub fn create_opaque_token() -> String {
    let mut bytes = [0u8; 32]; // 256 bits
    rand::thread_rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

pub fn decode_jwt_payload(jwt: &str) -> Outcome<Value> {
    let parts: Vec<&str> = jwt.splitn(3, '.').collect();

    if parts.len() < 2 {
        return Err(Errors::format(
            BadFormat::Received,
            "Error decoding jwt payload",
            None,
        ));
    }

    let payload = parts[1];
    let decoded = URL_SAFE_NO_PAD.decode(payload).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            "Error decoding jwt payload",
            Some(Box::new(e)),
        )
    })?;
    let value: Value = serde_json::from_slice(&decoded)?;

    Ok(value)
}
