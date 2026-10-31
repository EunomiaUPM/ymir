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

//! `NotFoundExt` and `ResourceError`: how a missing row becomes a 404.

use super::{Errors, Outcome};

/// Maps `None` to a missing resource error naming the entity and its id.
pub trait NotFoundExt<T> {
    fn or_not_found(self, id: impl std::fmt::Display, entity_name: &str) -> Outcome<T>;
}

impl<T> NotFoundExt<T> for Option<T> {
    fn or_not_found(self, id: impl std::fmt::Display, entity_name: &str) -> Outcome<T> {
        self.ok_or_else(|| ResourceError::not_found(id, entity_name))
    }
}

/// Factory for the usual 404.
pub struct ResourceError;

impl ResourceError {
    pub fn not_found(id: impl std::fmt::Display, entity_name: &str) -> Errors {
        Errors::missing_resource(id.to_string(), format!("{entity_name} not found"), None)
    }
}

#[cfg(test)]
mod tests;
