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

//! Why a presentation was rejected, as far as the holder may be told.

use crate::errors::{ErrorReason, Errors};

/// A rejection the verifier may tell the holder: something it could find out on its own. Any
/// other failure stays in the verifier's logs, so a forger learns nothing from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    Revoked,
    Suspended,
    Expired,
    NotYetValid,
    CredentialMissing,
}

impl Rejection {
    const ALL: [Rejection; 5] = [
        Rejection::Revoked,
        Rejection::Suspended,
        Rejection::Expired,
        Rejection::NotYetValid,
        Rejection::CredentialMissing,
    ];

    pub fn code(self) -> u32 {
        match self {
            Rejection::Revoked => 4410,
            Rejection::Suspended => 4411,
            Rejection::Expired => 4412,
            Rejection::NotYetValid => 4413,
            Rejection::CredentialMissing => 4414,
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Rejection::Revoked => "a credential is revoked",
            Rejection::Suspended => "a credential is suspended",
            Rejection::Expired => "a credential has expired",
            Rejection::NotYetValid => "a credential is not valid yet",
            Rejection::CredentialMissing => "a requested credential is missing",
        }
    }

    /// The error of this rejection; `path` says where, for the verifier's logs.
    pub fn error(self, path: impl Into<String>) -> Errors {
        Errors::validation_with(vec![ErrorReason {
            path: path.into(),
            code: self.code(),
            message: self.message().to_string(),
        }])
    }

    /// The rejection `error` carries, if any.
    pub fn of(error: &Errors) -> Option<Rejection> {
        error.info().reasons.iter().find_map(|reason| {
            Self::ALL
                .into_iter()
                .find(|rejection| rejection.code() == reason.code)
        })
    }

    /// What the holder may be told about `error`, if anything.
    pub fn public_reason(error: &Errors) -> Option<&'static str> {
        Self::of(error).map(Self::message)
    }
}
