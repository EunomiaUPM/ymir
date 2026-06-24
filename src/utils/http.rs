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

use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::http::header::{ACCEPT, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use axum::{Form, Json};
use reqwest::Response;
use serde::de::DeserializeOwned;
use urn::Urn;

use crate::errors::{BadFormat, Errors, Outcome, PetitionFailure};
use crate::types::gnap::grant_response::ErrorCode;

// ===== STRING & URI MANIPULATION =================================================================

/// Trims a fully qualified URI path layout back to its structural protocol base domain level.
pub fn trim_4_base(input: &str) -> String {
    let slashes: Vec<usize> = input.match_indices('/').map(|(i, _)| i).collect();

    if slashes.len() < 3 {
        return input.to_string();
    }

    let cut_index = slashes[2];

    input[..cut_index].to_string()
}

// ===== HTTP HEADER BUILDERS ======================================================================

/// Allocates an optimized standard HTTP [`HeaderMap`] initialized with standard application/json headers.
pub fn json_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers
}

// ===== ASYNC NETWORK RESPONSE EXTENSIONS =========================================================

/// Extended asynchronous trait provisioning high-level deserialization shortcuts over network raw [`Response`] objects.
#[async_trait]
pub trait ResponseExt {
    /// Deserializes the target network wire packet payload safely into structural model representations `T`.
    async fn parse_json<T: DeserializeOwned>(self) -> Outcome<T>;
    /// Consumes the wire packet context completely, yielding a raw text payload representation.
    async fn parse_text(self) -> Outcome<String>;
}

#[async_trait]
impl ResponseExt for Response {
    async fn parse_json<T: DeserializeOwned>(self) -> Outcome<T> {
        let url = self.url().to_string();
        let status = self.status();
        self.json().await.map_err(|e| {
            Errors::petition(
                &url,
                "unknown",
                Some(status),
                PetitionFailure::BodyDeserialization,
                "Error deserializing body",
                Some(Box::new(e)),
            )
        })
    }

    async fn parse_text(self) -> Outcome<String> {
        let url = self.url().to_string();
        let status = self.status();
        self.text().await.map_err(|e| {
            Errors::petition(
                &url,
                "unknown",
                Some(status),
                PetitionFailure::BodyRead,
                "Failed to read body",
                Some(Box::new(e)),
            )
        })
    }
}

// ===== AXUM EXTRACTOR LAYER UNWRAPPERS ===========================================================

/// Safely unwraps inbound Axum extract json vectors, converting framework errors into internal [`Errors::FormatError`].
pub fn extract_payload<T>(payload: Result<Json<T>, JsonRejection>) -> Outcome<T> {
    payload.map(|Json(v)| v).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            "Error extracting Json payload",
            Some(Box::new(e)),
        )
    })
}

/// Safely unwraps inbound Axum extract form parameters, converting errors into internal framework errors.
pub fn extract_form_payload<T>(payload: Result<Form<T>, FormRejection>) -> Outcome<T> {
    payload.map(|Form(v)| v).map_err(|e| {
        Errors::format(
            BadFormat::Received,
            "Error extracting form payload",
            Some(Box::new(e)),
        )
    })
}

/// Dispatches localized key parsing queries over standard unstructured query hash matrixes.
pub fn extract_query_param(params: &HashMap<String, String>, key: &str) -> Outcome<String> {
    params.get(key).cloned().ok_or_else(|| {
        Errors::format(
            BadFormat::Received,
            format!("Unable to retrieve '{}' from query params", key),
            None,
        )
    })
}

// ===== AUTHORIZATION TOKEN PROCESSING ============================================================

/// Validates perimeter headers to parse detached GNAP capability token values.
pub fn extract_gnap_token(headers: &HeaderMap) -> Outcome<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("GNAP "))
        .map(|token| token.to_string())
        .ok_or_else(|| Errors::unauthorized("GNAP token missing", None))
}

/// Validates perimeter headers to parse standard OAuth2 style Bearer authorization strings.
pub fn extract_bearer_token(headers: &HeaderMap) -> Outcome<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|token| token.to_string())
        .ok_or_else(|| Errors::unauthorized("Bearer token missing", None))
}

/// Parses unstructured network payload parameters safely into strongly typed data space [`Urn`] footprints.
pub fn extract_path_urn(urn: &String) -> Outcome<Urn> {
    let id_urn =
        Urn::from_str(urn).map_err(|e| Errors::parse(format!("Invalid Urn: {}", e), None))?;
    Ok(id_urn)
}

// ===== TRUST FRAMEWORK SPECIFIC CONVERSIONS ======================================================

/// Translates systemic internal Rust domain errors into formalized external GNAP payload error protocols.
pub fn errors_to_error_code(e: &Errors) -> ErrorCode {
    match e {
        // Client-side: Malformed or unmappable inputs
        Errors::FormatError { .. } => ErrorCode::InvalidRequest,
        Errors::ParseError { .. } => ErrorCode::InvalidRequest,
        Errors::FeatureNotImplError { .. } => ErrorCode::InvalidRequest,

        // Client-side: Invalid cryptographic boundary assertions
        Errors::UnauthorizedError { .. } => ErrorCode::InvalidClient,
        Errors::SecurityError { .. } => ErrorCode::InvalidClient,

        // Client-side: Valid identities explicitly denied by policies
        Errors::ForbiddenError { .. } => ErrorCode::RequestDenied,
        Errors::MissingActionError { .. } => ErrorCode::RequestDenied,

        // Client-side: Unknown target identifiers
        Errors::MissingResourceError { .. } => ErrorCode::InvalidRequest,

        // Server-side: Opaque database or internal infrastructure failure boundaries
        Errors::DatabaseError { .. }
        | Errors::ReadError { .. }
        | Errors::WriteError { .. }
        | Errors::VaultError { .. }
        | Errors::EnvVarError { .. }
        | Errors::ModuleNotActiveError { .. }
        | Errors::CrazyError { .. } => ErrorCode::Other("server_error".to_string()),

        // Server-side: Errors encountered when executing external network data boundaries
        Errors::PetitionError { .. }
        | Errors::ProviderError { .. }
        | Errors::ConsumerError { .. }
        | Errors::AuthorityError { .. }
        | Errors::WalletError { .. } => ErrorCode::Other("upstream_error".to_string()),
    }
}