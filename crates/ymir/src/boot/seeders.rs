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

//! One-off boot tasks (seeding, provisioning, cache resets) run in registration order.

#[cfg(feature = "redis")]
use crate::errors::Errors;
use crate::errors::Outcome;

/// When a seeder runs relative to the workers starting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootPhase {
    /// Before any worker is spawned; nothing is being served yet.
    BeforeServe,
    /// Once HTTP/gRPC listen, for seeders that call the agent's own API.
    AfterServe,
}

/// One-off task run at every boot, in the phase it asks for.
#[async_trait::async_trait]
pub trait BootSeeder: Send + Sync {
    /// Name shown in the boot logs.
    fn name(&self) -> &'static str;

    /// `AfterServe` unless the seeder must run before anything listens.
    fn phase(&self) -> BootPhase {
        BootPhase::AfterServe
    }

    /// Runs the task; an error aborts the boot.
    async fn seed(&self) -> Outcome<()>;
}

/// Empties the Redis cache so no stale entry outlives a restart.
#[cfg(feature = "redis")]
pub struct RedisCacheFlush {
    url: String,
}

#[cfg(feature = "redis")]
impl RedisCacheFlush {
    /// `url` is the full Redis URL, credentials included.
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Runs `FLUSHALL` on the Redis at `url`.
    pub async fn flush(url: &str) -> Outcome<()> {
        let client = redis::Client::open(url)
            .map_err(|err| Errors::crazy("Redis client open url failed", Some(Box::new(err))))?;
        let mut con = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|err| {
                Errors::crazy("Redis getting async connection failed", Some(Box::new(err)))
            })?;
        redis::cmd("FLUSHALL")
            .query_async::<()>(&mut con)
            .await
            .map_err(|err| Errors::crazy("Redis command failed", Some(Box::new(err))))
    }
}

#[cfg(feature = "redis")]
#[async_trait::async_trait]
impl BootSeeder for RedisCacheFlush {
    fn name(&self) -> &'static str {
        "redis-cache-flush"
    }

    fn phase(&self) -> BootPhase {
        BootPhase::BeforeServe
    }

    /// A cache that cannot be flushed is not fatal: log and keep booting.
    async fn seed(&self) -> Outcome<()> {
        match Self::flush(&self.url).await {
            Ok(()) => tracing::info!("Redis cache at {} flushed", self.url),
            Err(e) => tracing::warn!("Failed to flush Redis at {}: {}", self.url, e),
        }
        Ok(())
    }
}
