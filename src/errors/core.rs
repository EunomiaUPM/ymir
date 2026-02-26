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

use super::{AnyError, ErrorInfo, HttpContext, MissingAction, PetitionFailure};
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Errors {
    // HTTP CONTEXT
    PetitionError {
        info: ErrorInfo,
        ctx: HttpContext,
        failure: PetitionFailure,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    WalletError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    ProviderError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    ConsumerError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    AuthorityError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    // ACTION
    MissingActionError {
        info: ErrorInfo,
        action: MissingAction,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    // ID
    MissingResourceError {
        info: ErrorInfo,
        resource_id: String,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    // PATHS
    ReadError {
        info: ErrorInfo,
        path: String,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    WriteError {
        info: ErrorInfo,
        path: String,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    // BASICS
    FormatError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    UnauthorizedError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    ForbiddenError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    SecurityError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    DatabaseError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    FeatureNotImplError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    EnvVarError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    ModuleNotActiveError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    ParseError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    VaultError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    CrazyError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
}

impl Display for Errors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
            Errors::CrazyError { info, .. } => write!(f, "{}", info.message),
        }
    }
}

impl std::error::Error for Errors {}
