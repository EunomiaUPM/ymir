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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::LazyLock;
use std::time::Duration;

use crate::services::client::ClientService;

// ===== STATIC RUNTIME INSTANCES ==================================================================

/// Process-wide pool for request/response calls: 10 s per call (retries included), one
/// retry when idempotent, at most 16 in-flight calls per host and a breaker per host.
static CLIENT_SERVICE: LazyLock<ClientService> = LazyLock::new(|| {
    ClientService::builder()
        .concurrency(32)
        .per_host_concurrency(16)
        .deadline(Duration::from_secs(10))
        .max_retries(1)
        .circuit_breaker(5, Duration::from_secs(30))
        .build()
});

/// Pool for proxies and long-lived streams: no overall deadline, tuned keep-alive.
static STREAM_CLIENT_SERVICE: LazyLock<ClientService> = LazyLock::new(|| {
    ClientService::builder()
        .concurrency(32)
        .timeout(None)
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_keepalive(Duration::from_secs(60))
        .build()
});

// ===== SUBSYSTEM HOOKS ===========================================================================

/// Yields a static reference to the shared global [`ClientService`] management infrastructure.
pub fn http_client() -> &'static ClientService {
    &CLIENT_SERVICE
}

/// Shared [`ClientService`] for proxied and streamed traffic; pair it with `ClientTrait::stream`.
pub fn stream_client() -> &'static ClientService {
    &STREAM_CLIENT_SERVICE
}
