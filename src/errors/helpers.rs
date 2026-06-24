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

use axum::http::StatusCode;
use tracing::error;

use super::{ErrorInfo, Errors, HttpContext};

impl Errors {
    /// Factory builder initializing unified HTTP tracing context frames.
    pub fn build_ctx<R: Into<String>, S: Into<String>>(
        http_code: Option<StatusCode>,
        url: R,
        method: S,
    ) -> HttpContext {
        HttpContext {
            http_code,
            url: url.into(),
            method: method.into(),
        }
    }

    /// Builder pattern adapter to overwrite or inject raw context description extensions into [`ErrorInfo`].
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

    /// Reflective extraction yielding access to the shared core metadata [`ErrorInfo`].
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

    /// Pulls and prints structural diagnostic metrics concerning raw HTTP transaction attempts.
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
            Some(code) => format!("Http Code: {}", code),
            None => "".to_string(),
        };
        format!(
            "Url: {} \nMethod: {} \n{} \n",
            ctx.url, ctx.method, http_code
        )
    }

    /// Extract diagnostic tags mapping protocol execution failures.
    pub fn failure(&self) -> String {
        match self {
            Errors::PetitionError { failure, .. } => format!("Failure: {} \n", failure),
            _ => "".to_string(),
        }
    }

    /// Pulls explicit operational blockages from missing state requirements.
    pub fn action(&self) -> String {
        match self {
            Errors::MissingActionError { action, .. } => format!("Action: {} \n", action),
            _ => "".to_string(),
        }
    }

    /// Resolves structural asset indicators dropped or missing inside repositories.
    pub fn id(&self) -> String {
        match self {
            Errors::MissingResourceError { resource_id, .. } => {
                format!("Resource ID: {} \n", resource_id)
            }
            _ => "".to_string(),
        }
    }

    /// Extracts structural path context vectors from serialization or filesystem operations.
    pub fn path(&self) -> String {
        let path = match self {
            Errors::ReadError { path, .. } => path,
            Errors::WriteError { path, .. } => path,
            _ => return "".to_string(),
        };
        format!("Path: {} \n", path)
    }

    /// Gathers technical debugging stacks combining backtraces and dynamic standard error boxes.
    pub fn rest(&self) -> String {
        let (reason, source, backtrace) = match self {
            Errors::PetitionError { reason, source, backtrace, .. }
            | Errors::WalletError { reason, source, backtrace, .. }
            | Errors::ProviderError { reason, source, backtrace, .. }
            | Errors::ConsumerError { reason, source, backtrace, .. }
            | Errors::AuthorityError { reason, source, backtrace, .. }
            | Errors::MissingActionError { reason, source, backtrace, .. }
            | Errors::MissingResourceError { reason, source, backtrace, .. }
            | Errors::ReadError { reason, source, backtrace, .. }
            | Errors::WriteError { reason, source, backtrace, .. }
            | Errors::FormatError { reason, source, backtrace, .. }
            | Errors::UnauthorizedError { reason, source, backtrace, .. }
            | Errors::ForbiddenError { reason, source, backtrace, .. }
            | Errors::SecurityError { reason, source, backtrace, .. }
            | Errors::DatabaseError { reason, source, backtrace, .. }
            | Errors::FeatureNotImplError { reason, source, backtrace, .. }
            | Errors::EnvVarError { reason, source, backtrace, .. }
            | Errors::ModuleNotActiveError { reason, source, backtrace, .. }
            | Errors::ParseError { reason, source, backtrace, .. }
            | Errors::VaultError { reason, source, backtrace, .. }
            | Errors::CrazyError { reason, source, backtrace, .. } => (reason, source, backtrace),
        };

        let reason = format!("Reason: {}", reason);
        let source = match source {
            Some(data) => format!("Source: {}", data),
            None => "".to_string(),
        };

        format!("{} \n{} \n{} \n", reason, source, backtrace)
    }

    /// Direct reference accessor targeting the core technical description string slice.
    pub fn reason(&self) -> &str {
        match self {
            Errors::PetitionError { reason, .. }
            | Errors::WalletError { reason, .. }
            | Errors::ProviderError { reason, .. }
            | Errors::ConsumerError { reason, .. }
            | Errors::AuthorityError { reason, .. }
            | Errors::MissingActionError { reason, .. }
            | Errors::MissingResourceError { reason, .. }
            | Errors::ReadError { reason, .. }
            | Errors::WriteError { reason, .. }
            | Errors::FormatError { reason, .. }
            | Errors::UnauthorizedError { reason, .. }
            | Errors::ForbiddenError { reason, .. }
            | Errors::SecurityError { reason, .. }
            | Errors::DatabaseError { reason, .. }
            | Errors::FeatureNotImplError { reason, .. }
            | Errors::EnvVarError { reason, .. }
            | Errors::ModuleNotActiveError { reason, .. }
            | Errors::ParseError { reason, .. }
            | Errors::VaultError { reason, .. }
            | Errors::CrazyError { reason, .. } => reason.as_str(),
        }
    }

    /// Emits a structured log dump matching standard tracking envelopes to the active system logger.
    pub fn log(&self) {
        error!(
            "Error occurred: {}{}{}{}{}{}{}",
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

// ===== INBOUND ECOSYSTEM STANDARD TRAIT CONVERSIONS ==============================================

impl From<serde_json::Error> for Errors {
    fn from(e: serde_json::Error) -> Self {
        Errors::parse(e.to_string(), Some(Box::new(e)))
    }
}

impl From<urn::Error> for Errors {
    fn from(e: urn::Error) -> Self {
        Errors::parse(e.to_string(), Some(Box::new(e)))
    }
}