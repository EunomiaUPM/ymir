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

mod core;
mod did_builder;
mod did_doc;
mod did_service;
mod did_type;
mod jwk;
pub mod kid;
mod ver_method;
mod web;

pub use core::Did;
pub use did_builder::*;
pub use did_doc::DidDocument;
pub use did_service::*;
pub use did_type::*;
pub use jwk::{JwkDid, JwkDidConfig};
pub use kid::Kid;
pub use ver_method::*;
pub use web::{WebDid, WebDidConfig};
