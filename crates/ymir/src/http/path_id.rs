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

//! Ids (DIDs) in route paths, encoded as base64url without padding (RFC 4648 §5).
//!
//! A `did:web` with a port carries it percent-encoded (`did:web:host%3A3000`), and every
//! percent-decoding on the way (axum, a proxy, a gateway) turns it into `host:3000`, a different
//! DID. Base64url uses only `A-Z a-z 0-9 - _`, which no decoding changes. Browsers encode with
//! `encodePathId` from `@eunomia/ui`.

use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;

use crate::errors::{BadFormat, Errors, Outcome};
use crate::utils::{decode_url_safe_no_pad, encode_url_safe_no_pad};

/// The id behind the single path parameter of a route, already decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathId(pub String);

impl PathId {
    /// Path segment for `id`.
    pub fn encode(id: impl AsRef<[u8]>) -> String {
        encode_url_safe_no_pad(id)
    }

    /// The id behind `segment`; bad-format error if it is not base64url or not UTF-8 text.
    pub fn decode(segment: &str) -> Outcome<String> {
        let bytes = decode_url_safe_no_pad(segment).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "path id is not base64url",
                Some(Box::new(e)),
            )
        })?;
        String::from_utf8(bytes).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "path id is not UTF-8 text",
                Some(Box::new(e)),
            )
        })
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<S: Send + Sync> FromRequestParts<S> for PathId {
    type Rejection = Errors;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(segment) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                Errors::format(BadFormat::Received, "missing path id", Some(Box::new(e)))
            })?;
        Ok(Self(Self::decode(&segment)?))
    }
}

#[cfg(test)]
mod tests;
