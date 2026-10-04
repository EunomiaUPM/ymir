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

//! Axum middleware that turns a request's bearer token into the caller's `UserInfo`.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use crate::errors::{AppResult, Errors, Outcome};

use crate::services::token_validator::TokenValidatorTrait;

/// Authenticates the requests of a router: reads the bearer token (header, or `token` /
/// `access_token` query parameter), asks the validator for the user and stores it in the request,
/// where handlers take it as a `UserInfo` extractor.
#[derive(Clone)]
pub struct OauthHttpMiddleware {
    validator: Option<Arc<dyn TokenValidatorTrait>>,
    strict: bool,
}

impl OauthHttpMiddleware {
    /// `strict` rejects requests without a valid token; otherwise they pass unauthenticated.
    pub fn new(validator: Option<Arc<dyn TokenValidatorTrait>>, strict: bool) -> Self {
        Self { validator, strict }
    }

    /// Creates a strict auth middleware with a required token validator.
    pub fn strict(validator: Arc<dyn TokenValidatorTrait>) -> Self {
        Self::new(Some(validator), true)
    }

    /// Creates a permissive auth middleware where unauthenticated requests pass through.
    pub fn permissive(validator: Option<Arc<dyn TokenValidatorTrait>>) -> Self {
        Self::new(validator, false)
    }

    /// Axum middleware function extracting and validating token from state.
    pub async fn run(
        State(validator): State<Arc<dyn TokenValidatorTrait>>,
        mut req: Request,
        next: Next,
    ) -> AppResult<Response> {
        let token = Self::extract_token(&req);
        let user = validator.validate_token(token.as_deref()).await?;
        req.extensions_mut().insert(user);
        let mut resp = next.run(req).await;
        Self::apply_security_headers(&mut resp);
        Ok(resp)
    }

    /// Extracts bearer token from Authorization header or URL query string (`token`/`access_token`).
    pub fn extract_token(req: &Request) -> Option<String> {
        if let Ok(token) = Self::bearer(req.headers()) {
            return Some(token.to_string());
        }

        if let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    if (k == "token" || k == "access_token") && !v.trim().is_empty() {
                        return Some(v.to_string());
                    }
                }
            }
        }

        None
    }

    /// Extracts the Bearer token string from the HTTP Authorization header.
    pub fn bearer(headers: &HeaderMap) -> Outcome<&str> {
        headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Errors::unauthorized("missing or malformed Authorization header", None))
    }

    /// Middleware handler injecting security headers and validating identity.
    pub async fn handle(&self, mut req: Request, next: Next) -> Response {
        if let Some(validator) = &self.validator {
            let token = Self::extract_token(&req);
            match validator.validate_token(token.as_deref()).await {
                Ok(user) => {
                    req.extensions_mut().insert(user);
                }
                Err(e) if self.strict => {
                    return Self::unauthorized_response(&format!("invalid token: {e}"));
                }
                Err(_) => {}
            }
        }
        let mut resp = next.run(req).await;
        Self::apply_security_headers(&mut resp);
        resp
    }

    /// Applies standard protective HTTP security headers to the response.
    pub fn apply_security_headers(resp: &mut Response) {
        let headers = resp.headers_mut();
        headers.insert(
            "x-content-type-options",
            HeaderValue::from_static("nosniff"),
        );
        headers.insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
        headers.insert(
            "x-xss-protection",
            HeaderValue::from_static("1; mode=block"),
        );
        headers.insert(
            "referrer-policy",
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );
    }

    /// Builds a standardized 401 Unauthorized response with security headers.
    pub fn unauthorized_response(msg: &str) -> Response {
        let body = axum::Json(json!({
            "error": "unauthorized",
            "error_description": msg
        }));
        let mut resp = (StatusCode::UNAUTHORIZED, body).into_response();
        resp.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Bearer realm=\"ds-gateway\", error=\"invalid_token\""),
        );
        Self::apply_security_headers(&mut resp);
        resp
    }
}
