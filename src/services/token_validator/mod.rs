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

//! Turning the bearer token of a request into the user behind it.

mod fixed;
mod proxied;
mod token_validator_trait;

pub use fixed::FixedUserValidator;
pub use proxied::ProxiedTokenValidator;
#[cfg(feature = "mock")]
pub use token_validator_trait::MockOauthTokenValidatorTrait;
pub use token_validator_trait::OauthTokenValidatorTrait;
