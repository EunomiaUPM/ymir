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

use std::backtrace::Backtrace;
use axum::http::StatusCode;
use super::{AnyError, BadFormat, ErrorInfo, Errors, MissingAction, PetitionFailure};

impl Errors {
    /// Factory constructor processing downstream network failure metrics. Maps distinct error ranges per category.
    pub fn petition(
        url: impl Into<String>,
        method: impl Into<String>,
        http_code: Option<StatusCode>,
        failure: PetitionFailure,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        let (status_code, error_code) = match &failure {
            PetitionFailure::Network => (StatusCode::BAD_GATEWAY, 1100),
            PetitionFailure::HttpStatus(_) => (StatusCode::BAD_GATEWAY, 1200),
            PetitionFailure::BodyDeserialization => (StatusCode::BAD_GATEWAY, 1300),
            PetitionFailure::BodyRead => (StatusCode::BAD_GATEWAY, 1600),
            PetitionFailure::Serialization => (StatusCode::INTERNAL_SERVER_ERROR, 1400),
            PetitionFailure::Concurrency => (StatusCode::SERVICE_UNAVAILABLE, 1500),
        };

        Errors::PetitionError {
            info: ErrorInfo {
                message: "Petition Error".to_string(),
                error_code,
                status_code,
                details: None,
            },
            ctx: Errors::build_ctx(http_code, url, method),
            failure,
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Factory constructor managing failure boundaries occurring inside Decentralized Wallet engines.
    pub fn wallet(
        url: impl Into<String>,
        method: impl Into<String>,
        http_code: Option<StatusCode>,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        Errors::WalletError {
            info: ErrorInfo {
                message: "Wallet Error".to_string(),
                error_code: 1200,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Factory constructor processing operational failures under the Provider role context.
    pub fn provider(
        url: impl Into<String>,
        method: impl Into<String>,
        http_code: Option<StatusCode>,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        Errors::ProviderError {
            info: ErrorInfo {
                message: "Provider Error".to_string(),
                error_code: 1300,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Shorthand constructor tracking failures at the inbound Provider Grant endpoint.
    pub fn provider_grant(reason: impl Into<String>) -> Self {
        Errors::ProviderError {
            info: ErrorInfo {
                message: "Provider Error".to_string(),
                error_code: 1300,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
            },
            ctx: Errors::build_ctx(None, "Provider Grant Endpoint", "POST"),
            reason: reason.into(),
            source: None,
            backtrace: Backtrace::capture(),
        }
    }

    /// Factory constructor processing operational failures under the Consumer role context.
    pub fn consumer(
        url: impl Into<String>,
        method: impl Into<String>,
        http_code: Option<StatusCode>,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        Errors::ConsumerError {
            info: ErrorInfo {
                message: "Consumer Error".to_string(),
                error_code: 1400,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Factory constructor tracking infrastructure or handshake failures inside external Authorization Servers.
    pub fn authority(
        url: impl Into<String>,
        method: impl Into<String>,
        http_code: Option<StatusCode>,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        Errors::AuthorityError {
            info: ErrorInfo {
                message: "Authority Error".to_string(),
                error_code: 1500,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Shorthand constructor targeting transactional authorization loops at raw AS continuation endpoints.
    pub fn authority_grant(reason: impl Into<String>) -> Self {
        Errors::AuthorityError {
            info: ErrorInfo {
                message: "Authority Error".to_string(),
                error_code: 1500,
                status_code: StatusCode::BAD_GATEWAY,
                details: None,
            },
            ctx: Errors::build_ctx(None, "Authority Grant Endpoint", "POST"),
            reason: reason.into(),
            source: None,
            backtrace: Backtrace::capture(),
        }
    }

    /// Factory constructor tracking state prerequisites or setup capabilities missing during protocol evaluations.
    pub fn missing_action(
        action: MissingAction,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        let error_code = match action {
            MissingAction::Token => 3110,
            MissingAction::Wallet => 3120,
            MissingAction::Key => 3130,
            MissingAction::Did => 3140,
            MissingAction::Onboarding => 3150,
            _ => 3100,
        };
        Errors::MissingActionError {
            info: ErrorInfo {
                message: "Missing Action Error".to_string(),
                error_code,
                status_code: StatusCode::PRECONDITION_FAILED,
                details: None,
            },
            action,
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Factory constructor mapping entity isolation mismatches inside repositories.
    pub fn missing_resource(
        id: impl Into<String>,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        Errors::MissingResourceError {
            info: ErrorInfo {
                message: "Missing Resource Error".to_string(),
                error_code: 3200,
                status_code: StatusCode::NOT_FOUND,
                details: None,
            },
            resource_id: id.into(),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Factory constructor checking schema structural validation mismatches.
    pub fn format(option: BadFormat, reason: impl Into<String>, source: Option<AnyError>) -> Self {
        let (error_code, status_code) = match option {
            BadFormat::Sent => (3110, StatusCode::BAD_GATEWAY),
            BadFormat::Received => (3120, StatusCode::BAD_REQUEST),
            _ => (3100, StatusCode::BAD_REQUEST),
        };
        Errors::FormatError {
            info: ErrorInfo {
                message: "Format Error".to_string(),
                error_code,
                status_code,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Standard identity validation tracking builder.
    pub fn unauthorized(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::UnauthorizedError {
            info: ErrorInfo {
                message: "Unauthorized Error".to_string(),
                error_code: 4200,
                status_code: StatusCode::UNAUTHORIZED,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Resource access privilege denial tracking builder.
    pub fn forbidden(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::ForbiddenError {
            info: ErrorInfo {
                message: "Forbidden Error".to_string(),
                error_code: 4300,
                status_code: StatusCode::FORBIDDEN,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Cryptographic verify, decryption, or message tracking integrity error factory.
    pub fn security(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::SecurityError {
            info: ErrorInfo {
                message: "Security Error".to_string(),
                error_code: 4400,
                status_code: StatusCode::UNPROCESSABLE_ENTITY,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Standard internal database mapping tracker.
    pub fn db(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::DatabaseError {
            info: ErrorInfo {
                message: "Database Error".to_string(),
                error_code: 5100,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Stubs development tracking engine constructor.
    pub fn not_impl(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::FeatureNotImplError {
            info: ErrorInfo {
                message: "Feature Not Implemented".to_string(),
                error_code: 5200,
                status_code: StatusCode::NOT_IMPLEMENTED,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Runtime application setting extraction failure constructor.
    pub fn env_var(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::EnvVarError {
            info: ErrorInfo {
                message: "Environment Variable Error".to_string(),
                error_code: 5300,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Dynamic configurations operational module gating check failure builder.
    pub fn not_active(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::ModuleNotActiveError {
            info: ErrorInfo {
                message: "Module Not Active Error".to_string(),
                error_code: 5400,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// File read pipeline error factory.
    pub fn read(
        path: impl Into<String>,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        Errors::ReadError {
            info: ErrorInfo {
                message: "Read Error".to_string(),
                error_code: 5510,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
            },
            path: path.into(),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// File write pipeline serialization error factory.
    pub fn write(
        path: impl Into<String>,
        reason: impl Into<String>,
        source: Option<AnyError>,
    ) -> Self {
        Errors::WriteError {
            info: ErrorInfo {
                message: "Write Error".to_string(),
                error_code: 5520,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
            },
            path: path.into(),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Data transformations parsing evaluation step failure tracker.
    pub fn parse(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::ParseError {
            info: ErrorInfo {
                message: "Parse Error".to_string(),
                error_code: 5530,
                status_code: StatusCode::BAD_REQUEST,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Specialized validation failure factory tailored for Data Space component templates.
    pub fn validation(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        let reason = reason.into();
        Errors::ParseError {
            info: ErrorInfo {
                message: "Error validating connector template".to_string(),
                error_code: 5531,
                status_code: StatusCode::BAD_REQUEST,
                details: Some(reason.clone()),
            },
            reason,
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// Hardware enclave or cryptographic Key Vault subsystem failure tracking builder.
    pub fn vault(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::VaultError {
            info: ErrorInfo {
                message: "Vault Error".to_string(),
                error_code: 5600,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }

    /// High level irregular fallback tracking panic-equivalent entry point factory.
    pub fn crazy(reason: impl Into<String>, source: Option<AnyError>) -> Self {
        Errors::CrazyError {
            info: ErrorInfo {
                message: "Something unexpected happened".to_string(),
                error_code: 6000,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None,
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture(),
        }
    }
}