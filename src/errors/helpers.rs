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

use axum::http::StatusCode;
use super::{ErrorInfo, Errors, HttpContext};
use tracing::error;

impl Errors {
    pub fn build_ctx<R: Into<String>, S: Into<String>>(
        http_code: Option<StatusCode>,
        url: R,
        method: S,
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
            Errors::CrazyError { info, .. } => info.details = details,
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
            Errors::CrazyError { info, .. } => info,
        }
    }
    pub fn context(&self) -> String {
        let ctx = match self {
            Errors::PetitionError { ctx, .. } => ctx,
            Errors::WalletError { ctx, .. } => ctx,
            Errors::ProviderError { ctx, .. } => ctx,
            Errors::ConsumerError { ctx, .. } => ctx,
            Errors::AuthorityError { ctx, .. } => ctx,
            _ => return "".to_string(),
        };

        let http_code = match ctx.http_code {
            Some(code) => {
                format!("Http Code: {}", code)
            }
            None => "".to_string(),
        };
        format!("Url: {} \n Method: {} \n {}", ctx.url, ctx.method, http_code)
    }
    pub fn failure(&self) -> String {
        match self {
            Errors::PetitionError { failure, .. } => format!("Failure: {}", failure),
            _ => "".to_string(),
        }
    }
    pub fn action(&self) -> String {
        match self {
            Errors::MissingActionError { action, .. } => format!("Action: {}", action),
            _ => "".to_string(),
        }
    }
    pub fn id(&self) -> String {
        match self {
            Errors::MissingResourceError { resource_id, .. } => {
                format!("Resource ID: {}", resource_id)
            }
            _ => "".to_string(),
        }
    }
    pub fn path(&self) -> String {
        let path = match self {
            Errors::ReadError { path, .. } => path,
            Errors::WriteError { path, .. } => path,
            _ => return "".to_string(),
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
            Errors::CrazyError { reason, source, backtrace, .. } => (reason, source, backtrace),
        };

        let reason = format!("Reason: {}", reason);

        let source = match source {
            Some(data) => format!("Source: {}", data),
            None => "".to_string(),
        };

        format!("{} \n {} \n {}", reason, source, backtrace)
    }

    pub fn log(&self) {
        error!(
            "Error occurred: {}\nContext:\n{}\n {}\n {}\n {}\n {}\n {}",
            self,
            self.context(),
            self.failure(),
            self.action(),
            self.id(),
            self.path(),
            self.rest(),
        );
    }
}
