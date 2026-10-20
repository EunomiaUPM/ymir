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

//! Something that passed its checks. Only [`Verified::verify`] builds one, so a `Verified<T>`
//! always means that `T`'s own [`Verifiable::check`] succeeded.

use std::ops::Deref;

use async_trait::async_trait;
use serde::Serialize;

use crate::errors::Outcome;

/// A value that can be checked: signature, audience, shape… whatever makes it trustworthy.
#[async_trait]
pub trait Verifiable: Sized + Send + Sync {
    /// What the check needs besides the value (expected audience, nonce…).
    type Context: Send + Sync;

    async fn check(&self, ctx: &Self::Context) -> Outcome<()>;
}

/// `T` once its check passed. No public constructor and no `Deserialize`, so it cannot be forged.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct Verified<T>(T);

impl<T: Verifiable> Verified<T> {
    /// The only way to get a `Verified`: runs `value`'s check and wraps it if it passes.
    pub async fn verify(value: T, ctx: &T::Context) -> Outcome<Self> {
        value.check(ctx).await?;
        Ok(Verified(value))
    }
}

impl<T> Verified<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Verified<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}
