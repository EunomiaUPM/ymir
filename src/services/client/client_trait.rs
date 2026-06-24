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

use async_trait::async_trait;
use axum::http::HeaderMap;
use reqwest::Response;

use crate::errors::Outcome;
use crate::types::http::HttpBody;

/// Abstract Asynchronous HTTP Client interface.
///
/// Provides a unified contract for executing network petitions across data spaces,
/// managing raw responses and isolating business logic from specific HTTP runtimes.
#[async_trait]
pub trait ClientTrait: Send + Sync {
    /// Executes an HTTP GET request against the target URL.
    async fn get(&self, url: &str, headers: Option<HeaderMap>) -> Outcome<Response>;

    /// Executes an HTTP POST request transmitting the specified operational payload.
    async fn post(
        &self,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response>;

    /// Executes an HTTP PUT request to modify target cloud resources.
    async fn put(&self, url: &str, headers: Option<HeaderMap>, body: HttpBody)
    -> Outcome<Response>;

    /// Executes an HTTP DELETE request to remove remote transactional assets.
    async fn delete(
        &self,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response>;
}
