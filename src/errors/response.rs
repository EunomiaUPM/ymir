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

use axum::Json;
use axum::response::{IntoResponse, Response};

use super::Errors;

/// Axum network boundary translation mapping [`Errors`] to wire-level responses.
///
/// Ensures every application-level failure triggers automated downstream structured logging 
/// before serializing the inner [`ErrorInfo`] to network boundaries via JSON payloads.
impl IntoResponse for Errors {
    fn into_response(self) -> Response {
        // Enforces asynchronous structural trace dumping to the tracing subsystem subscriber.
        self.log();

        let mut info = self.info().clone();

        // Fallback boundary: guarantees client payloads always obtain precise context 
        // by hoisting the inner 'reason' string if no explicit 'details' are set.
        if info.details.is_none() {
            info.details = Some(self.reason().to_string());
        }
        let status = info.status_code;

        // Marshals response structures directly into standard Axum tuples.
        (status, Json(info)).into_response()
    }
}