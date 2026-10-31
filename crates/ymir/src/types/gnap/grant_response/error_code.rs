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

use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::impl_serde_via_str;

#[derive(Debug, Clone)]
pub enum ErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidInteraction,
    InvalidRotation,
    InvalidContinuation,
    UserDenied,
    RequestDenied,
    UnknownUser,
    UnknownInteraction,
    TooFast,
    TooManyAttempts,
    Other(String),
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ErrorCode::InvalidRequest => "invalid_request",
            ErrorCode::InvalidClient => "invalid_client",
            ErrorCode::InvalidInteraction => "invalid_interaction",
            ErrorCode::InvalidRotation => "invalid_rotation",
            ErrorCode::InvalidContinuation => "invalid_continuation",
            ErrorCode::UserDenied => "user_denied",
            ErrorCode::RequestDenied => "request_denied",
            ErrorCode::UnknownUser => "unknown_user",
            ErrorCode::UnknownInteraction => "unknown_interaction",
            ErrorCode::TooFast => "too_fast",
            ErrorCode::TooManyAttempts => "too_many_attempts",
            ErrorCode::Other(other) => other.as_str(),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for ErrorCode {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "invalid_request" => Ok(ErrorCode::InvalidRequest),
            "invalid_client" => Ok(ErrorCode::InvalidClient),
            "invalid_interaction" => Ok(ErrorCode::InvalidInteraction),
            "invalid_rotation" => Ok(ErrorCode::InvalidRotation),
            "invalid_continuation" => Ok(ErrorCode::InvalidContinuation),
            "user_denied" => Ok(ErrorCode::UserDenied),
            "request_denied" => Ok(ErrorCode::RequestDenied),
            "unknown_user" => Ok(ErrorCode::UnknownUser),
            "unknown_interaction" => Ok(ErrorCode::UnknownInteraction),
            "too_fast" => Ok(ErrorCode::TooFast),
            "too_many_attempts" => Ok(ErrorCode::TooManyAttempts),
            _ => Ok(ErrorCode::Other(s.to_string())),
        }
    }
}

impl_serde_via_str!(ErrorCode);
