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

use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter, Result};

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

use crate::errors::sub_errors::{ErrorInfo, HttpContext};
use crate::types::errors::{BadFormat, MissingAction};

#[derive(Debug)]
pub enum Errors {
    // HTTP CONTEXT
    PetitionError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    WalletError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    ProviderError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    ConsumerError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    AuthorityError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    // ACTION
    MissingActionError {
        info: ErrorInfo,
        action: MissingAction,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    // ID
    MissingResourceError {
        info: ErrorInfo,
        resource_id: String,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    // PATHS
    ReadError {
        info: ErrorInfo,
        path: String,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    WriteError {
        info: ErrorInfo,
        path: String,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    // BASICS
    FormatError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    UnauthorizedError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    ForbiddenError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    SecurityError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    DatabaseError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    FeatureNotImplError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    EnvVarError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    ModuleNotActiveError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    ParseError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    VaultError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    },
    CrazyError {
        info: ErrorInfo,
        reason: String,
        source: Option<anyhow::Error>,
        backtrace: Backtrace
    }
}

impl Display for Errors {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Errors::PetitionError { info, .. } => write!(f, "{}", info.message),
            Errors::WalletError { info, .. } => write!(f, "{}", info.message),
            Errors::ProviderError { info, .. } => write!(f, "{}", info.message),
            Errors::ConsumerError { info, .. } => write!(f, "{}", info.message),
            Errors::AuthorityError { info, .. } => write!(f, "{}", info.message),
            Errors::MissingActionError { info, .. } => write!(f, "{}", info.message),
            Errors::MissingResourceError { info, .. } => write!(f, "{}", info.message),
            Errors::FormatError { info, .. } => write!(f, "{}", info.message),
            Errors::UnauthorizedError { info, .. } => write!(f, "{}", info.message),
            Errors::ForbiddenError { info, .. } => write!(f, "{}", info.message),
            Errors::SecurityError { info, .. } => write!(f, "{}", info.message),
            Errors::DatabaseError { info, .. } => write!(f, "{}", info.message),
            Errors::FeatureNotImplError { info, .. } => write!(f, "{}", info.message),
            Errors::EnvVarError { info, .. } => write!(f, "{}", info.message),
            Errors::ModuleNotActiveError { info, .. } => write!(f, "{}", info.message),
            Errors::ReadError { info, .. } => write!(f, "{}", info.message),
            Errors::WriteError { info, .. } => write!(f, "{}", info.message),
            Errors::ParseError { info, .. } => write!(f, "{}", info.message),
            Errors::VaultError { info, .. } => write!(f, "{}", info.message),
            Errors::CrazyError { info, .. } => write!(f, "{}", info.message)
        }
    }
}

impl std::error::Error for Errors {}

impl Errors {
    pub fn petition<R: Into<String>, S: Into<String>, T: Into<String>>(
        url: R,
        method: S,
        http_code: Option<u16>,
        reason: T,
        source: Option<anyhow::Error>
    ) -> Self {
        Errors::PetitionError {
            info: ErrorInfo {
                message: "Petition Error".to_string(),
                error_code: 1100,
                status_code: StatusCode::BAD_GATEWAY,
                details: None
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn wallet<R: Into<String>, S: Into<String>, T: Into<String>>(
        url: R,
        method: S,
        http_code: Option<u16>,
        reason: T,
        source: Option<anyhow::Error>
    ) -> Self {
        Errors::WalletError {
            info: ErrorInfo {
                message: "Wallet Error".to_string(),
                error_code: 1200,
                status_code: StatusCode::BAD_GATEWAY,
                details: None
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn provider<R: Into<String>, S: Into<String>, T: Into<String>>(
        url: R,
        method: S,
        http_code: Option<u16>,
        reason: T,
        source: Option<anyhow::Error>
    ) -> Self {
        Errors::ProviderError {
            info: ErrorInfo {
                message: "Provider Error".to_string(),
                error_code: 1300,
                status_code: StatusCode::BAD_GATEWAY,
                details: None
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn consumer<R: Into<String>, S: Into<String>, T: Into<String>>(
        url: R,
        method: S,
        http_code: Option<u16>,
        reason: T,
        source: Option<anyhow::Error>
    ) -> Self {
        Errors::ConsumerError {
            info: ErrorInfo {
                message: "Consumer Error".to_string(),
                error_code: 1400,
                status_code: StatusCode::BAD_GATEWAY,
                details: None
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn authority<R: Into<String>, S: Into<String>, T: Into<String>>(
        url: R,
        method: S,
        http_code: Option<u16>,
        reason: T,
        source: Option<anyhow::Error>
    ) -> Self {
        Errors::AuthorityError {
            info: ErrorInfo {
                message: "Authority Error".to_string(),
                error_code: 1500,
                status_code: StatusCode::BAD_GATEWAY,
                details: None
            },
            ctx: Errors::build_ctx(http_code, url, method),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn missing_action<R: Into<String>>(
        action: MissingAction,
        reason: R,
        source: Option<anyhow::Error>
    ) -> Self {
        let error_code = match action {
            MissingAction::Token => 3110,
            MissingAction::Wallet => 3120,
            MissingAction::Key => 3130,
            MissingAction::Did => 3140,
            MissingAction::Onboarding => 3150,
            _ => 3100
        };
        Errors::MissingActionError {
            info: ErrorInfo {
                message: "Missing Action Error".to_string(),
                error_code,
                status_code: StatusCode::PRECONDITION_FAILED,
                details: None
            },
            action,
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn missing_resource<R: Into<String>, T: Into<String>>(
        id: R,
        reason: T,
        source: Option<anyhow::Error>
    ) -> Self {
        Errors::MissingResourceError {
            info: ErrorInfo {
                message: "Missing Resource Error".to_string(),
                error_code: 3200,
                status_code: StatusCode::NOT_FOUND,
                details: None
            },
            resource_id: id.into(),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn format<R: Into<String>>(
        option: BadFormat,
        reason: R,
        source: Option<anyhow::Error>
    ) -> Self {
        let (error_code, status_code) = match option {
            BadFormat::Sent => (3110, StatusCode::BAD_GATEWAY),
            BadFormat::Received => (3120, StatusCode::BAD_REQUEST),
            _ => (3100, StatusCode::BAD_REQUEST)
        };
        Errors::FormatError {
            info: ErrorInfo {
                message: "Format Error".to_string(),
                error_code,
                status_code,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn unauthorized<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::UnauthorizedError {
            info: ErrorInfo {
                message: "Unauthorized Error".to_string(),
                error_code: 4200,
                status_code: StatusCode::UNAUTHORIZED,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn forbidden<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::ForbiddenError {
            info: ErrorInfo {
                message: "Forbidden Error".to_string(),
                error_code: 4300,
                status_code: StatusCode::FORBIDDEN,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn security<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::SecurityError {
            info: ErrorInfo {
                message: "Security Error".to_string(),
                error_code: 4400,
                status_code: StatusCode::UNPROCESSABLE_ENTITY,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn db<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::DatabaseError {
            info: ErrorInfo {
                message: "Database Error".to_string(),
                error_code: 5100,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn not_impl<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::FeatureNotImplError {
            info: ErrorInfo {
                message: "Feature Not Implemented".to_string(),
                error_code: 5200,
                status_code: StatusCode::NOT_IMPLEMENTED,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn env_var<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::EnvVarError {
            info: ErrorInfo {
                message: "Environment Variable Error".to_string(),
                error_code: 5300,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn not_active<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::ModuleNotActiveError {
            info: ErrorInfo {
                message: "Module Not Active Error".to_string(),
                error_code: 5400,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn read<R: Into<String>, S: Into<String>>(
        path: R,
        reason: S,
        source: Option<anyhow::Error>
    ) -> Self {
        Errors::ReadError {
            info: ErrorInfo {
                message: "Read Error".to_string(),
                error_code: 5510,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None
            },
            path: path.into(),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn write<R: Into<String>, S: Into<String>>(
        path: R,
        reason: S,
        source: Option<anyhow::Error>
    ) -> Self {
        Errors::WriteError {
            info: ErrorInfo {
                message: "Write Error".to_string(),
                error_code: 5520,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None
            },
            path: path.into(),
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn parse<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::ParseError {
            info: ErrorInfo {
                message: "Parse Error".to_string(),
                error_code: 5530,
                status_code: StatusCode::BAD_REQUEST,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn vault<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::VaultError {
            info: ErrorInfo {
                message: "Vault Error".to_string(),
                error_code: 5600,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
    pub fn crazy<R: Into<String>>(reason: R, source: Option<anyhow::Error>) -> Self {
        Errors::VaultError {
            info: ErrorInfo {
                message: "Something unexpected happened".to_string(),
                error_code: 6000,
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                details: None
            },
            reason: reason.into(),
            source,
            backtrace: Backtrace::capture()
        }
    }
}

// ERROR HELPERS
impl Errors {
    fn build_ctx<R: Into<String>, S: Into<String>>(
        http_code: Option<u16>,
        url: R,
        method: S
    ) -> HttpContext {
        HttpContext { http_code, url: url.into(), method: method.into() }
    }

    pub fn with_details<S: Into<String>>(mut self, details: S) -> Self {
        let details = Some(details.into());
        match &mut self {
            Errors::PetitionError { info, .. } => info.details = details,
            Errors::WalletError { info, .. } => info.details = details,
            Errors::ProviderError { info, .. } => info.details = details,
            Errors::ConsumerError { info, .. } => info.details = details,
            Errors::AuthorityError { info, .. } => info.details = details,
            Errors::MissingActionError { info, .. } => info.details = details,
            Errors::MissingResourceError { info, .. } => info.details = details,
            Errors::ReadError { info, .. } => info.details = details,
            Errors::WriteError { info, .. } => info.details = details,
            Errors::FormatError { info, .. } => info.details = details,
            Errors::UnauthorizedError { info, .. } => info.details = details,
            Errors::ForbiddenError { info, .. } => info.details = details,
            Errors::SecurityError { info, .. } => info.details = details,
            Errors::DatabaseError { info, .. } => info.details = details,
            Errors::FeatureNotImplError { info, .. } => info.details = details,
            Errors::EnvVarError { info, .. } => info.details = details,
            Errors::ModuleNotActiveError { info, .. } => info.details = details,
            Errors::ParseError { info, .. } => info.details = details,
            Errors::VaultError { info, .. } => info.details = details,
            Errors::CrazyError { info, .. } => info.details = details
        }
        self
    }
    pub fn info(&self) -> &ErrorInfo {
        match self {
            Errors::PetitionError { info, .. } => info,
            Errors::WalletError { info, .. } => info,
            Errors::ProviderError { info, .. } => info,
            Errors::ConsumerError { info, .. } => info,
            Errors::AuthorityError { info, .. } => info,
            Errors::MissingActionError { info, .. } => info,
            Errors::MissingResourceError { info, .. } => info,
            Errors::ReadError { info, .. } => info,
            Errors::WriteError { info, .. } => info,
            Errors::FormatError { info, .. } => info,
            Errors::UnauthorizedError { info, .. } => info,
            Errors::ForbiddenError { info, .. } => info,
            Errors::SecurityError { info, .. } => info,
            Errors::DatabaseError { info, .. } => info,
            Errors::FeatureNotImplError { info, .. } => info,
            Errors::EnvVarError { info, .. } => info,
            Errors::ModuleNotActiveError { info, .. } => info,
            Errors::ParseError { info, .. } => info,
            Errors::VaultError { info, .. } => info,
            Errors::CrazyError { info, .. } => info
        }
    }
    pub fn context(&self) -> String {
        let ctx = match self {
            Errors::PetitionError { ctx, .. } => ctx,
            Errors::WalletError { ctx, .. } => ctx,
            Errors::ProviderError { ctx, .. } => ctx,
            Errors::ConsumerError { ctx, .. } => ctx,
            Errors::AuthorityError { ctx, .. } => ctx,
            _ => return "".to_string()
        };

        let http_code = match ctx.http_code {
            Some(code) => {
                format!("Http Code: {}", code)
            }
            None => "".to_string()
        };
        format!("Url: {} \n Method: {} \n {}", ctx.url, ctx.method, http_code)
    }
    pub fn action(&self) -> String {
        match self {
            Errors::MissingActionError { action, .. } => format!("Action: {}", action),
            _ => "".to_string()
        }
    }
    pub fn id(&self) -> String {
        match self {
            Errors::MissingResourceError { resource_id, .. } => {
                format!("Resource ID: {}", resource_id)
            }
            _ => "".to_string()
        }
    }
    pub fn path(&self) -> String {
        let path = match self {
            Errors::ReadError { path, .. } => path,
            Errors::WriteError { path, .. } => path,
            _ => return "".to_string()
        };
        format!("Path: {}", path)
    }
    pub fn rest(&self) -> String {
        let (reason, source, backtrace) = match self {
            Errors::PetitionError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::WalletError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::ProviderError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::ConsumerError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::AuthorityError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::MissingActionError { reason, source, backtrace, .. } => {
                (reason, source, backtrace)
            }
            Errors::MissingResourceError { reason, source, backtrace, .. } => {
                (reason, source, backtrace)
            }
            Errors::ReadError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::WriteError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::FormatError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::UnauthorizedError { reason, source, backtrace, .. } => {
                (reason, source, backtrace)
            }
            Errors::ForbiddenError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::SecurityError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::DatabaseError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::FeatureNotImplError { reason, source, backtrace, .. } => {
                (reason, source, backtrace)
            }
            Errors::EnvVarError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::ModuleNotActiveError { reason, source, backtrace, .. } => {
                (reason, source, backtrace)
            }
            Errors::ParseError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::VaultError { reason, source, backtrace, .. } => (reason, source, backtrace),
            Errors::CrazyError { reason, source, backtrace, .. } => (reason, source, backtrace)
        };

        let reason = format!("Reason: {}", reason);

        let source = match source {
            Some(data) => format!("Source: {}", data),
            None => "".to_string()
        };

        format!("{} \n {} \n {}", reason, source, backtrace)
    }
}

impl IntoResponse for Errors {
    fn into_response(self) -> Response {
        self.log();

        let info = self.info();
        let status = info.status_code;

        (status, Json(info)).into_response()
    }
}

impl Errors {
    pub fn log(&self) {
        error!(
            "Error occurred: {}\nContext:\n{}\nAction: {}\nID: {}\nPath: {}\nRest:\n{}",
            self,
            self.context(),
            self.action(),
            self.id(),
            self.path(),
            self.rest(),
        );
    }
}
