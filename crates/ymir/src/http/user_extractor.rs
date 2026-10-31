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

//! The authenticated user as an axum extractor.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::errors::Errors;
use crate::types::oauth::UserInfo;

/// Reads the [`UserInfo`] an auth middleware stored in the request extensions; without one the
/// route was not authenticated and the request is rejected with a 401.
impl<S: Send + Sync> FromRequestParts<S> for UserInfo {
    type Rejection = Errors;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserInfo>()
            .cloned()
            .ok_or_else(|| Errors::unauthorized("authentication required: missing user", None))
    }
}
