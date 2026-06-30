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
use std::fmt::{Display, Formatter};

use super::{AnyError, ErrorInfo, HttpContext, MissingAction, PetitionFailure};

/// Centralized Operational Error Taxonomy for the Identity and Data Space Ecosystem.
///
/// Implements [`std::error::Error`] and provides structural variants capturing rich multiprotocol
/// contexts, raw IO state tracks, dynamic source errors ([`AnyError`]), and automated stack [`Backtrace`] frames.
#[derive(Debug)]
pub enum Errors {
    // ===== HTTP & ECOSYSTEM CONTEXT ERRORS =======================================================
    /// Triggered when an outbound or inbound HTTP network request/handshake transaction fails.
    PetitionError {
        info: ErrorInfo,
        ctx: HttpContext,
        failure: PetitionFailure,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Internal failure context originating from the core SSI Wallet subsystem operations.
    WalletError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Data Space error context occurring when operating under the **Provider** role.
    ProviderError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Data Space error context occurring when operating under the **Consumer** role.
    ConsumerError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Authorization Server or trust anchor domain validation error context.
    AuthorityError {
        info: ErrorInfo,
        ctx: HttpContext,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },

    // ===== PROTOCOL LIFECYCLE ERRORS ============================================================
    /// Occurs when a mandatory GNAP or OAuth transactional action state is requested but missing.
    MissingActionError {
        info: ErrorInfo,
        action: MissingAction,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Occurs when a specific relational or unique entity resource cannot be resolved by its identifier.
    MissingResourceError {
        info: ErrorInfo,
        resource_id: String,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },

    // ===== FILESYSTEM & STORAGE IO ERRORS ========================================================
    /// Read file-system IO block operations failure.
    ReadError {
        info: ErrorInfo,
        path: String,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Write/Serialization file-system IO block operations failure.
    WriteError {
        info: ErrorInfo,
        path: String,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },

    // ===== FOUNDATIONAL SECURITY & PLATFORM BASE ERRORS ==========================================
    /// Data structures schema mismatch or unexpected envelope formats.
    FormatError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Client request lacks proper credentials or identity authentication indicators.
    UnauthorizedError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Client identity is authenticated but lacks required access privileges for the resource.
    ForbiddenError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Cryptographic validation or message signature verification failure.
    SecurityError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Internal engine database operational error originating from the Sea-ORM layer.
    DatabaseError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Executed code pathways pointing to non-implemented features or architectural stubs.
    FeatureNotImplError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Missing or corrupted environment variable declarations during host boot sequences.
    EnvVarError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Execution requested over a business logic module that has been flagged as inactive in configs.
    ModuleNotActiveError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Data transformations, serialization/deserialization, or string parsing failure steps.
    ParseError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Internal secure hardware enclave or Vault Service infrastructure failure.
    VaultError {
        info: ErrorInfo,
        reason: String,
        source: Option<AnyError>,
        backtrace: Backtrace,
    },
    /// Fallback variant for unclassified, highly irregular, or panic-equivalent edge-cases.
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
            Errors::PetitionError { info, .. } => write!(f, "{}\n", info.message),
            Errors::WalletError { info, .. } => write!(f, "{}\n", info.message),
            Errors::ProviderError { info, .. } => write!(f, "{}\n", info.message),
            Errors::ConsumerError { info, .. } => write!(f, "{}\n", info.message),
            Errors::AuthorityError { info, .. } => write!(f, "{}\n", info.message),
            Errors::MissingActionError { info, .. } => write!(f, "{}\n", info.message),
            Errors::MissingResourceError { info, .. } => write!(f, "{}\n", info.message),
            Errors::FormatError { info, .. } => write!(f, "{}\n", info.message),
            Errors::UnauthorizedError { info, .. } => write!(f, "{}\n", info.message),
            Errors::ForbiddenError { info, .. } => write!(f, "{}\n", info.message),
            Errors::SecurityError { info, .. } => write!(f, "{}\n", info.message),
            Errors::DatabaseError { info, .. } => write!(f, "{}\n", info.message),
            Errors::FeatureNotImplError { info, .. } => write!(f, "{}\n", info.message),
            Errors::EnvVarError { info, .. } => write!(f, "{}\n", info.message),
            Errors::ModuleNotActiveError { info, .. } => write!(f, "{}\n", info.message),
            Errors::ReadError { info, .. } => write!(f, "{}\n", info.message),
            Errors::WriteError { info, .. } => write!(f, "{}\n", info.message),
            Errors::ParseError { info, reason, .. } => {
                if f.alternate() {
                    write!(f, "{}: {}", info.message, reason)
                } else {
                    write!(f, "{}", info.message)
                }
            }
            Errors::VaultError { info, .. } => write!(f, "{}\n", info.message),
            Errors::CrazyError { info, .. } => write!(f, "{}\n", info.message),
        }
    }
}

impl std::error::Error for Errors {}
