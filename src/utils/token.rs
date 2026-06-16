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

pub fn create_opaque_token() -> String {
    let mut bytes = [0u8; 32]; // 256 bits
    rand::thread_rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

pub fn is_active(iat: i64) -> Outcome<()> {
    let now = Utc::now().timestamp();
    if now >= iat {
        Ok(())
    } else {
        Err(Errors::forbidden("Token is not yet valid", None))
    }
}

pub fn has_expired(exp: i64) -> Outcome<()> {
    let now = Utc::now().timestamp();
    if now <= exp {
        Ok(())
    } else {
        Err(Errors::forbidden("Token has expired", None))
    }
}
