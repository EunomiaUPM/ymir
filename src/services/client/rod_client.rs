/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::http::HeaderMap;
use reqwest::{Client, RequestBuilder, Response};
use tokio::sync::Semaphore;

use crate::errors::{Errors, Outcome, PetitionFailure};
use crate::services::client::ClientTrait;
use crate::types::http::Body;

pub struct ClientService {
    client: Client,
    limiter: Arc<Semaphore>,
    max_retries: u32,
}

impl Default for ClientService {
    fn default() -> Self {
        Self::new(10, 10, 3)
    }
}

impl ClientService {
    pub fn new(concurrency_limit: usize, timeout_secs: u64, max_retries: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(concurrency_limit)
            .build()
            .expect("Failed to build request client");

        Self { client, limiter: Arc::new(Semaphore::new(concurrency_limit)), max_retries }
    }

    // -----------------------------------------------------------------------
    // INTERNALS
    // -----------------------------------------------------------------------

    async fn dispatch(
        &self,
        method: reqwest::Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: Body,
    ) -> Outcome<Response> {
        let _permit = self.limiter.acquire().await.map_err(|_| {
            Errors::petition(
                url,
                method.as_str(),
                None,
                PetitionFailure::Concurrency,
                "Semaphore closed",
                None,
            )
        })?;

        self.execute_with_retries(method, url, headers, body).await
    }

    async fn execute_with_retries(
        &self,
        method: reqwest::Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: Body,
    ) -> Outcome<Response> {
        let mut attempt = 1;

        loop {
            match self.send_request(method.clone(), url, headers.clone(), body.clone()).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    if !self.should_retry(&err, attempt) {
                        return Err(err);
                    }
                    let backoff = Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(backoff).await;
                    attempt += 1;
                }
            }
        }
    }

    fn should_retry(&self, err: &Errors, attempt: u32) -> bool {
        if attempt > self.max_retries {
            return false;
        }
        match err {
            Errors::PetitionError { failure, .. } => match failure {
                PetitionFailure::Network => true,
                PetitionFailure::HttpStatus(s) => s.is_server_error(),
                _ => false,
            },
            _ => false,
        }
    }

    async fn send_request(
        &self,
        method: reqwest::Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: Body,
    ) -> Outcome<Response> {
        let mut req = self.client.request(method.clone(), url);

        if let Some(h) = headers {
            req = req.headers(h);
        }

        req = self.apply_body(req, body)?;

        let response = req.send().await.map_err(|e| {
            Errors::petition(
                url,
                method.as_str(),
                e.status().map(|s| s),
                PetitionFailure::Network,
                "Error sending petition",
                Some(Box::new(e)),
            )
        })?;

        if response.status().is_server_error() {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            return Err(Errors::petition(
                url,
                method.as_str(),
                Some(status.clone()),
                PetitionFailure::HttpStatus(status),
                message,
                None,
            ));
        }

        Ok(response)
    }

    fn apply_body(&self, req: RequestBuilder, body: Body) -> Outcome<RequestBuilder> {
        let req = match body {
            Body::Json(value) => req.json(&value),
            Body::Raw(s) => req.body(s),
            Body::Form(pairs) => match serde_urlencoded::to_string(&pairs) {
                Ok(encoded) => req
                    .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(encoded),
                Err(e) => return Err(Errors::parse("Unable to parse form", Some(Box::new(e)))),
            },
            Body::None => req,
        };
        Ok(req)
    }
}

#[async_trait]
impl ClientTrait for ClientService {
    async fn get(&self, url: &str, headers: Option<HeaderMap>) -> Outcome<Response> {
        self.dispatch(reqwest::Method::GET, url, headers, Body::None).await
    }

    async fn post(&self, url: &str, headers: Option<HeaderMap>, body: Body) -> Outcome<Response> {
        self.dispatch(reqwest::Method::POST, url, headers, body).await
    }

    async fn put(&self, url: &str, headers: Option<HeaderMap>, body: Body) -> Outcome<Response> {
        self.dispatch(reqwest::Method::PUT, url, headers, body).await
    }

    async fn delete(&self, url: &str, headers: Option<HeaderMap>, body: Body) -> Outcome<Response> {
        self.dispatch(reqwest::Method::DELETE, url, headers, body).await
    }
}
