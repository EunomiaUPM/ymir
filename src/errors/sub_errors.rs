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

use std::fmt::{Display, Formatter};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

// =================================================================================================
// SUB_ERRORS STRUCTS & ENUMS
// =================================================================================================

/// Standardized JSON response payload transmitted to remote network clients upon failure.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorInfo {
    /// Human-readable high-level message summarizing the classification of the fault.
    pub message: String,
    /// Internal business error code assigned to target this specific technical domain variant.
    pub error_code: u16,
    /// Associated HTTP boundary network layer response code. Skipped during JSON translation.
    #[serde(skip)]
    pub status_code: StatusCode,
    /// Enriched operational context, debugging insights, or underlying message breakdowns.
    pub details: Option<String>,
}

/// HTTP network interaction metrics tracking the targeted perimeter endpoint properties.
#[derive(Debug, Clone)]
pub struct HttpContext {
    /// The actual HTTP status code returned by the remote server, if available.
    pub http_code: Option<StatusCode>,
    /// Target destination URL string evaluated during execution.
    pub url: String,
    /// Outbound standard REST action method string identifier (e.g., `POST`, `GET`).
    pub method: String,
}

/// Classifies the mechanical failure root-cause for outbound HTTP requests dispatched via network clients.
#[derive(Debug)]
pub enum PetitionFailure {
    /// Client transport link disconnection or target host unreachable (`reqwest::Error`).
    Network,
    /// The remote platform responded successfully but flagged an invalid non-2xx status code.
    HttpStatus(StatusCode),
    /// Inbound payload syntax mismatch while parsing remote buffer outputs into structural models.
    BodyDeserialization,
    /// Active network stream interruption or chunk allocation timeout while sucking down text bytes.
    BodyRead,
    /// Failed to serialize request bodies or structural payloads into data envelopes.
    Serialization,
    /// Multi-threaded internal rate-limiter or synchronization backpressure semaphore blockades.
    Concurrency,
}

impl Display for PetitionFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PetitionFailure::Network => write!(f, "Network failure"),
            PetitionFailure::HttpStatus(code) => write!(f, "Remote HTTP error {}", code),
            PetitionFailure::BodyDeserialization => write!(f, "Deserialization failed"),
            PetitionFailure::BodyRead => write!(f, "Failed to read response body"),
            PetitionFailure::Serialization => write!(f, "Serialization failed"),
            PetitionFailure::Concurrency => write!(f, "Concurrency limit reached"),
        }
    }
}

/// Identifies mandatory prerequisite configuration blocks missing during dynamic business execution routes.
#[derive(Serialize, Deserialize, Debug)]
pub enum MissingAction {
    /// OAuth2/GNAP Token sequence contexts missing or revoked.
    Token,
    /// Core wallet environment data records missing or uninitialized.
    Wallet,
    /// Local identity DID registries missing or unresolvable.
    Did,
    /// Cryptographic key references missing or deleted.
    Key,
    /// Host data space directory registration handshakes missing.
    Onboarding,
    /// Requested Verifiable Credentials properties absent from storage.
    Credentials,
    /// Unclassified prerequisite structural element target mismatch.
    Unknown,
}

impl Display for MissingAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MissingAction::Token => "Token",
            MissingAction::Wallet => "Wallet",
            MissingAction::Key => "Key",
            MissingAction::Did => "DID",
            MissingAction::Onboarding => "Onboarding",
            MissingAction::Credentials => "Credentials",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

/// Categorizes envelope validation failures concerning semantic data models.
pub enum BadFormat {
    /// Outbound request layout violates specification standards.
    Sent,
    /// Inbound external payload failed schema validation rules.
    Received,
    /// Unclassified format layout failure context.
    Unknown,
}