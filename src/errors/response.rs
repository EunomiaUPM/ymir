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

impl IntoResponse for Errors {
    fn into_response(self) -> Response {
        self.log();

        let mut info = self.info().clone();
        if info.details.is_none() {
            info.details = Some(self.reason().to_string());
        }
        let status = info.status_code;

        (status, Json(info)).into_response()
    }
}

impl Errors {}
