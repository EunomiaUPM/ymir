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

use crate::config::types::ApiConfig;
use crate::errors::Outcome;
use crate::utils::read;

/// Shared behavior for structural components managing core API gateway descriptors.
pub trait ApiConfigTrait {
    // ===== EXTRACTION ANCHORS ====================================================================

    /// Returns a backing reference to the root API configuration model.
    fn api(&self) -> &ApiConfig;

    // ===== METADATA & FILESYSTEM RESOLUTION ======================================================

    /// Dispatches a synchronous file-system read operation to recover the target OpenAPI schema raw string.
    ///
    /// # Errors
    /// Returns an [`Errors::ReadError`](crate::errors::Errors::ReadError) if the specified schema path matrix 
    /// cannot be resolved or accessed on the host system.
    fn get_openapi(&self) -> Outcome<String> {
        read(&self.api().openapi_path)
    }

    /// Assembles the canonical API version prefix route path.
    ///
    /// Yields a standard string layout matching the pattern: `/api/<version>`.
    fn get_api_version(&self) -> String {
        format!("/api/{}", self.api().version)
    }
}