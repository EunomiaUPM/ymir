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

mod constructors;
mod core;
mod helpers;
mod response;
mod sub_errors;

// Re-expose primary structural error representation entity.
pub use core::Errors;
pub use sub_errors::*;

use axum::response::Response;

/// Dynamic dispatch boundary type-alias mapping third-party standard library error wrappers.
pub type AnyError = Box<dyn std::error::Error + Send + Sync>;

/// Core operational result wrapper utilizing internal taxonomy engines for domain layer operations.
pub type Outcome<T> = Result<T, Errors>;

/// Perimeter HTTP interface wrapper matching standard Axum network routing architectures.
pub type AppResult<T = Response> = Result<T, Errors>;

/// Infrastructure conversion trait simplifying direct translation from repository level drivers.
///
/// Automatically implements default structural formatting mapping raw data layers
/// into structural application-wide [`Errors::DatabaseError`] targets.
pub trait RepoIntoErrors: std::error::Error + Send + Sync + 'static {
    /// Translates raw external database or layer failures into native tracking structures.
    fn into_errors(self) -> Errors
    where
        Self: Sized,
    {
        Errors::db(self.to_string(), Some(Box::new(self)))
    }
}
