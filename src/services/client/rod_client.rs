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

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::errors::{Errors, Outcome, PetitionFailure};
use crate::services::client::ClientTrait;
use crate::services::client::metrics::METRICS;
use crate::services::client::resilience::{Backoff, BreakerConfig, BreakerState, HostGuards};
use crate::services::client::trace_context::TraceContext;
use crate::types::http::{HttpBody, StreamBody};
use async_trait::async_trait;
use axum::http::HeaderMap;
use opentelemetry::KeyValue;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode, Url};
use tokio::sync::Semaphore;
use tracing::field::Empty;
use tracing::{Instrument, info};

/// Tuning knobs of a [`ClientService`]; build one with [`ClientService::builder`].
#[derive(Clone, Debug)]
pub struct ClientConfig {
    concurrency: usize,
    per_host_concurrency: Option<usize>,
    timeout: Option<Duration>,
    deadline: Option<Duration>,
    connect_timeout: Option<Duration>,
    max_retries: u32,
    backoff: Backoff,
    circuit_breaker: Option<BreakerConfig>,
    accept_invalid_certs: bool,
    pool_idle_timeout: Option<Duration>,
    tcp_keepalive: Option<Duration>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            per_host_concurrency: None,
            timeout: Some(Duration::from_secs(10)),
            deadline: None,
            connect_timeout: None,
            max_retries: 0,
            backoff: Backoff {
                base: Duration::from_millis(200),
                cap: Duration::from_secs(2),
            },
            circuit_breaker: None,
            accept_invalid_certs: false,
            pool_idle_timeout: None,
            tcp_keepalive: None,
        }
    }
}

impl ClientConfig {
    /// Maximum in-flight requests; also the idle pool size per host.
    pub fn concurrency(mut self, limit: usize) -> Self {
        self.concurrency = limit;
        self
    }

    /// In-flight requests any single host may hold (bulkhead); defaults to `concurrency`.
    pub fn per_host_concurrency(mut self, limit: usize) -> Self {
        self.per_host_concurrency = Some(limit);
        self
    }

    /// Per-attempt deadline; `None` lets long-lived streams (SSE, proxies) run unbounded.
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Budget of a whole call, queueing, retries and backoff included.
    pub fn deadline(mut self, deadline: Duration) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Retries of idempotent requests on network errors, `5xx` and `429`.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Full-jitter exponential backoff between retries, capped at `cap`.
    pub fn retry_backoff(mut self, base: Duration, cap: Duration) -> Self {
        self.backoff = Backoff { base, cap };
        self
    }

    /// Per-host breaker: `failure_threshold` consecutive failures open it for `open_for`.
    pub fn circuit_breaker(mut self, failure_threshold: u32, open_for: Duration) -> Self {
        self.circuit_breaker = Some(BreakerConfig {
            failure_threshold,
            open_for,
        });
        self
    }

    /// Trusts any TLS certificate; only for testing proxies reaching self-signed upstreams.
    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = Some(timeout);
        self
    }

    pub fn tcp_keepalive(mut self, interval: Duration) -> Self {
        self.tcp_keepalive = Some(interval);
        self
    }

    pub fn build(self) -> ClientService {
        let mut builder = Client::builder()
            .pool_max_idle_per_host(self.concurrency)
            .danger_accept_invalid_certs(self.accept_invalid_certs);
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(timeout) = self.connect_timeout {
            builder = builder.connect_timeout(timeout);
        }
        if let Some(timeout) = self.pool_idle_timeout {
            builder = builder.pool_idle_timeout(timeout);
        }
        if let Some(interval) = self.tcp_keepalive {
            builder = builder.tcp_keepalive(interval);
        }

        let per_host = self.per_host_concurrency.unwrap_or(self.concurrency);
        ClientService {
            client: builder.build().expect("Failed to build request client"),
            limiter: Arc::new(Semaphore::new(self.concurrency)),
            hosts: HostGuards::new(per_host, self.circuit_breaker),
            max_retries: self.max_retries,
            backoff: self.backoff,
            deadline: self.deadline,
        }
    }
}

/// A failed attempt plus the server's `Retry-After`, when it sent one.
struct AttemptError {
    error: Errors,
    retry_after: Option<Duration>,
}

impl From<Errors> for AttemptError {
    fn from(error: Errors) -> Self {
        Self {
            error,
            retry_after: None,
        }
    }
}

/// Rate-limited HTTP Client Service with per-host bulkheads, circuit breakers and
/// jittered retries.
///
/// Retries idempotent requests on network errors, `5xx` and `429`; any other status is
/// handed back untouched so callers can read error bodies. Every call runs in a client
/// span and carries the current W3C trace context.
pub struct ClientService {
    client: Client,
    limiter: Arc<Semaphore>,
    hosts: HostGuards,
    max_retries: u32,
    backoff: Backoff,
    deadline: Option<Duration>,
}

impl Default for ClientService {
    fn default() -> Self {
        ClientConfig::default().build()
    }
}

impl ClientService {
    pub fn new(concurrency_limit: usize, timeout_secs: u64, max_retries: u32) -> Self {
        Self::builder()
            .concurrency(concurrency_limit)
            .timeout(Some(Duration::from_secs(timeout_secs)))
            .max_retries(max_retries)
            .build()
    }

    pub fn builder() -> ClientConfig {
        ClientConfig::default()
    }

    /// Hosts whose circuit is open or probing.
    pub fn degraded_hosts(&self) -> Vec<(String, BreakerState)> {
        self.hosts.degraded()
    }

    // -----------------------------------------------------------------------
    // INTERNALS
    // -----------------------------------------------------------------------

    async fn dispatch(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response> {
        let span = Self::client_span(&method, url);
        let started = Instant::now();
        let mut attributes = vec![
            KeyValue::new("http.request.method", method.to_string()),
            KeyValue::new("server.address", Self::authority(url)),
        ];
        let outcome = self
            .execute_with_retries(method, url, headers, body)
            .instrument(span)
            .await;
        attributes.push(match &outcome {
            Ok(res) => KeyValue::new("http.response.status_code", res.status().as_u16() as i64),
            Err(Errors::PetitionError { failure, .. }) => {
                KeyValue::new("error.type", failure.to_string())
            }
            Err(_) => KeyValue::new("error.type", "other"),
        });
        METRICS
            .duration
            .record(started.elapsed().as_secs_f64(), &attributes);
        outcome
    }

    /// `url.full` drops the query string: it may carry tokens.
    fn client_span(method: &Method, url: &str) -> tracing::Span {
        let parsed = Url::parse(url).ok();
        let address = parsed
            .as_ref()
            .and_then(|u| u.host_str().map(str::to_owned))
            .unwrap_or_default();
        let port = parsed.as_ref().and_then(Url::port_or_known_default);
        let full = parsed
            .map(|mut u| {
                u.set_query(None);
                u.to_string()
            })
            .unwrap_or_default();
        tracing::info_span!(
            "http.client",
            otel.name = %method,
            otel.kind = "client",
            otel.status_code = Empty,
            http.request.method = %method,
            server.address = %address,
            server.port = port,
            url.full = %full,
            http.response.status_code = Empty,
            http.request.resend_count = Empty,
            circuit.state = Empty,
        )
    }

    /// Breakers and bulkheads are per `scheme://host:port`.
    fn authority(url: &str) -> String {
        match Url::parse(url) {
            Ok(u) => format!(
                "{}://{}:{}",
                u.scheme(),
                u.host_str().unwrap_or_default(),
                u.port_or_known_default().unwrap_or_default()
            ),
            Err(_) => url.to_string(),
        }
    }

    async fn execute_with_retries(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response> {
        let started = Instant::now();
        let guard = self.hosts.guard(&Self::authority(url));
        let mut headers = headers.unwrap_or_default();
        TraceContext::inject(&mut headers);
        let span = tracing::Span::current();
        let mut attempt = 1;

        loop {
            if attempt > 1 {
                span.record("http.request.resend_count", attempt - 1);
            }
            if let Some(breaker) = &guard.breaker {
                if !breaker.try_acquire() {
                    span.record("circuit.state", BreakerState::Open.as_str());
                    span.record("otel.status_code", "ERROR");
                    return Err(Self::circuit_open(&method, url));
                }
            }

            let remaining = self.remaining(started);
            let attempt_future = async {
                // Host bulkhead first, so a saturated host never holds a global permit.
                let _host = guard
                    .permits
                    .acquire()
                    .await
                    .map_err(|_| Self::semaphore_closed(&method, url))?;
                let _global = self.permit(&method, url).await?;
                self.send_request(method.clone(), url, headers.clone(), body.clone())
                    .await
            };
            let outcome = match remaining {
                Some(remaining) => tokio::time::timeout(remaining, attempt_future)
                    .await
                    .unwrap_or_else(|_| Err(Self::deadline_exceeded(&method, url).into())),
                None => attempt_future.await,
            };

            let err = match outcome {
                Ok(response) => {
                    if let Some(breaker) = &guard.breaker {
                        breaker.on_success();
                    }
                    span.record("http.response.status_code", response.status().as_u16());
                    return Ok(response);
                }
                Err(err) => err,
            };
            if let Some(breaker) = &guard.breaker {
                if Self::counts_against_host(&err.error) {
                    breaker.on_failure();
                } else {
                    breaker.on_success();
                }
                span.record("circuit.state", breaker.state().as_str());
            }

            let delay = err
                .retry_after
                .unwrap_or_else(|| self.backoff.delay(attempt));
            let fits = self.remaining(started).is_none_or(|left| delay < left);
            if !self.should_retry(&method, &err.error, attempt) || !fits {
                if let Errors::PetitionError {
                    failure: PetitionFailure::HttpStatus(status),
                    ..
                } = &err.error
                {
                    span.record("http.response.status_code", status.as_u16());
                }
                span.record("otel.status_code", "ERROR");
                return Err(err.error);
            }
            METRICS.retries.add(1, &[]);
            tokio::time::sleep(delay).await;
            attempt += 1;
        }
    }

    fn remaining(&self, started: Instant) -> Option<Duration> {
        self.deadline
            .map(|deadline| deadline.saturating_sub(started.elapsed()))
    }

    async fn permit(
        &self,
        method: &Method,
        url: &str,
    ) -> Outcome<tokio::sync::SemaphorePermit<'_>> {
        self.limiter
            .acquire()
            .await
            .map_err(|_| Self::semaphore_closed(method, url))
    }

    /// Only idempotent methods are retried: a lost POST response may already have acted.
    fn should_retry(&self, method: &Method, err: &Errors, attempt: u32) -> bool {
        if attempt > self.max_retries || !method.is_idempotent() {
            return false;
        }
        match err {
            Errors::PetitionError { failure, .. } => match failure {
                PetitionFailure::Network => true,
                PetitionFailure::HttpStatus(s) => Self::is_transient(*s),
                _ => false,
            },
            _ => false,
        }
    }

    /// Unreachable, timed out or `5xx`: the host itself is failing. A `429` is a live host.
    fn counts_against_host(err: &Errors) -> bool {
        match err {
            Errors::PetitionError { failure, .. } => match failure {
                PetitionFailure::Network => true,
                PetitionFailure::HttpStatus(s) => s.is_server_error(),
                _ => false,
            },
            _ => false,
        }
    }

    fn is_transient(status: StatusCode) -> bool {
        status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS
    }

    /// Delta-seconds form only; an HTTP-date is ignored and the backoff applies.
    fn retry_after(response: &Response) -> Option<Duration> {
        response
            .headers()
            .get(reqwest::header::RETRY_AFTER)?
            .to_str()
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()
            .map(Duration::from_secs)
    }

    async fn send_request(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        body: HttpBody,
    ) -> Result<Response, AttemptError> {
        info!("Sending {} to {}", method, url);
        let req = self.client.request(method.clone(), url).headers(headers);
        let req = self.apply_body(req, body)?;

        let response = req
            .send()
            .await
            .map_err(|e| Self::network_error(&method, url, e))?;

        if Self::is_transient(response.status()) {
            let status = response.status();
            let retry_after = Self::retry_after(&response);
            let message = response.text().await.unwrap_or_default();
            return Err(AttemptError {
                error: Errors::petition(
                    url,
                    method.as_str(),
                    Some(status),
                    PetitionFailure::HttpStatus(status),
                    message,
                    None,
                ),
                retry_after,
            });
        }

        Ok(response)
    }

    fn network_error(method: &Method, url: &str, e: reqwest::Error) -> Errors {
        Errors::petition(
            url,
            method.as_str(),
            e.status(),
            PetitionFailure::Network,
            "Error sending petition",
            Some(Box::new(e)),
        )
    }

    fn circuit_open(method: &Method, url: &str) -> Errors {
        Errors::petition(
            url,
            method.as_str(),
            None,
            PetitionFailure::CircuitOpen,
            "Circuit open: host is failing, call not attempted",
            None,
        )
    }

    fn deadline_exceeded(method: &Method, url: &str) -> Errors {
        Errors::petition(
            url,
            method.as_str(),
            None,
            PetitionFailure::Network,
            "Call deadline exceeded",
            None,
        )
    }

    fn semaphore_closed(method: &Method, url: &str) -> Errors {
        Errors::petition(
            url,
            method.as_str(),
            None,
            PetitionFailure::Concurrency,
            "Semaphore closed",
            None,
        )
    }

    fn apply_body(&self, req: RequestBuilder, body: HttpBody) -> Outcome<RequestBuilder> {
        let req = match body {
            HttpBody::Json(value) => req.json(&value),
            HttpBody::Raw(s) => req.body(s),
            HttpBody::Bytes(bytes) => req.body(bytes),
            HttpBody::Form(pairs) => match serde_urlencoded::to_string(&pairs) {
                Ok(encoded) => req
                    .header(
                        reqwest::header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(encoded),
                Err(e) => return Err(Errors::parse("Unable to parse form", Some(Box::new(e)))),
            },
            HttpBody::None => req,
        };
        Ok(req)
    }
}

#[async_trait]
impl ClientTrait for ClientService {
    async fn request(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: HttpBody,
    ) -> Outcome<Response> {
        self.dispatch(method, url, headers, body).await
    }

    /// Sent once, without retries or breaker (the body cannot be replayed), but traced.
    async fn stream(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: StreamBody,
        timeout: Option<Duration>,
    ) -> Outcome<Response> {
        let span = Self::client_span(&method, url);
        async {
            let _permit = self.permit(&method, url).await?;
            info!("Streaming {} to {}", method, url);
            let mut headers = headers.unwrap_or_default();
            TraceContext::inject(&mut headers);
            let mut req = self
                .client
                .request(method.clone(), url)
                .headers(headers)
                .body(body);
            if let Some(t) = timeout {
                req = req.timeout(t);
            }
            let response = req
                .send()
                .await
                .map_err(|e| Self::network_error(&method, url, e))?;
            tracing::Span::current()
                .record("http.response.status_code", response.status().as_u16());
            Ok(response)
        }
        .instrument(span)
        .await
    }
}
