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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

pub mod capabilities;
pub mod config;
pub mod data;
pub mod errors;
pub mod http;
mod macros;
pub mod modules;
pub mod services;
pub mod types;
pub mod utils;
pub mod secret;
pub mod registry;
pub mod json_schema;
pub mod http_tracing;
pub mod telemetry;
pub mod serde_utils;
pub mod id_mac;
pub mod paginated_spec;
pub mod query;
pub mod batch_requests;
pub mod vault_utils;
pub mod boot;
pub mod module_loader;
#[cfg(feature = "redis")]
pub mod cache;
pub mod http_global_404;
pub mod oauth;
#[cfg(feature = "grpc")]
pub mod grpc;
