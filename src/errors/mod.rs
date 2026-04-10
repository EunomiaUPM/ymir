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

pub use core::Errors;

use axum::response::Response;
pub use sub_errors::*;

pub type AnyError = Box<dyn std::error::Error + Send + Sync>;
pub type Outcome<T> = Result<T, Errors>;
pub type AppResult<T = Response> = Result<T, Errors>;

/// Trait for repository-specific error types to convert into [`Errors`].
///
/// Implement this trait (with an empty body) on any repo error enum that
/// derives `thiserror::Error` and whose fields are `Send + Sync + 'static`.
/// The default implementation wraps the error as a [`Errors::db`] variant.
pub trait RepoIntoErrors: std::error::Error + Send + Sync + 'static {
    fn into_errors(self) -> Errors
    where
        Self: Sized,
    {
        Errors::db(self.to_string(), Some(Box::new(self)))
    }
}
