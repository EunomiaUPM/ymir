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
use reqwest::Method;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::Outcome;
use crate::services::client::ClientTrait;
use crate::types::http::HttpBody;
use crate::utils::ResponseExt;

/// Strict shortcuts over [`ClientTrait`]: any non-2xx status becomes a petition error.
#[async_trait]
pub trait ClientExt: ClientTrait {
    async fn get_json<R>(&self, url: &str, headers: Option<HeaderMap>) -> Outcome<R>
    where
        R: DeserializeOwned + Send,
    {
        self.send_json(Method::GET, url, headers, HttpBody::None).await
    }

    async fn post_json<T, R>(&self, url: &str, headers: Option<HeaderMap>, body: &T) -> Outcome<R>
    where
        T: Serialize + Sync,
        R: DeserializeOwned + Send,
    {
        self.send_json(Method::POST, url, headers, HttpBody::json(body)?)
            .await
    }

    async fn put_json<T, R>(&self, url: &str, headers: Option<HeaderMap>, body: &T) -> Outcome<R>
    where
        T: Serialize + Sync,
        R: DeserializeOwned + Send,
    {
        self.send_json(Method::PUT, url, headers, HttpBody::json(body)?)
            .await
    }

    /// POST whose response body is ignored; fits endpoints that answer an empty 2xx.
    async fn post_ok(&self, url: &str, headers: Option<HeaderMap>, body: HttpBody) -> Outcome<()> {
        self.send_ok(Method::POST, url, headers, body).await
    }

    async fn delete_ok(&self, url: &str, headers: Option<HeaderMap>) -> Outcome<()> {
        self.send_ok(Method::DELETE, url, headers, HttpBody::None)
            .await
    }

    /// Any method and body, expecting a 2xx JSON answer.
    async fn send_json<R>(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<R>
    where
        R: DeserializeOwned + Send,
    {
        self.request(method, url, headers, body)
            .await?
            .ensure_success()
            .await?
            .parse_json()
            .await
    }

    /// Any method and body, expecting a 2xx whose body is discarded.
    async fn send_ok(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<()> {
        self.request(method, url, headers, body)
            .await?
            .ensure_success()
            .await?;
        Ok(())
    }
}

impl<C: ClientTrait + ?Sized> ClientExt for C {}
