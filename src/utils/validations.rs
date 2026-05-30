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

use chrono::Utc;
use serde_json::Value;

use crate::errors::{BadFormat, Errors, Outcome};

pub fn validate_data(node: &Value, field: &str) -> Outcome<String> {
    node.as_str()
        .ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                format!("Field '{}' is not a string", field),
                None,
            )
        })
        .map(|s| s.to_string())
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
