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

use std::time::Duration;

use async_trait::async_trait;
use axum::http::HeaderMap;
use reqwest::{Method, Response};

use crate::errors::Outcome;
use crate::types::http::{HttpBody, StreamBody};

/// Abstract Asynchronous HTTP Client interface.
///
/// Returns the raw [`Response`] for any status below `5xx`, so callers can read protocol
/// error bodies (e.g. GNAP); [`super::ClientExt`] adds strict, typed shortcuts on top.
#[async_trait]
pub trait ClientTrait: Send + Sync {
    /// Executes an HTTP request of any method, retrying transient failures when idempotent.
    async fn request(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response>;

    /// Sends a streamed body once (it cannot be replayed) and returns whatever status arrives.
    async fn stream(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: StreamBody,
        timeout: Option<Duration>,
    ) -> Outcome<Response>;

    async fn get(&self, url: &str, headers: Option<HeaderMap>) -> Outcome<Response> {
        self.request(Method::GET, url, headers, HttpBody::None).await
    }

    async fn post(
        &self,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response> {
        self.request(Method::POST, url, headers, body).await
    }

    async fn put(
        &self,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response> {
        self.request(Method::PUT, url, headers, body).await
    }

    async fn delete(
        &self,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response> {
        self.request(Method::DELETE, url, headers, body).await
    }
}
