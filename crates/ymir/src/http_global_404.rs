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

//! Fallback handler for unknown routes, answering 404 in the common error format.
//!
//! The boot sets it as the fallback of the composed router; modules do not mount it.

use axum::response::IntoResponse;
use crate::errors::Errors;

/// Logs the unknown route and answers 404.
pub async fn global_handler_404(uri: axum::http::Uri) -> impl IntoResponse {
    tracing::info!("404 Not Found: {}", uri);
    Errors::missing_resource(uri.to_string(), "Route not found or Method not allowed", None)
        .into_response()
}
