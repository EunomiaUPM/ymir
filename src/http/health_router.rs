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

use axum::Router;
use axum::response::IntoResponse;
use axum::routing::get;

/// HTTP API Gateway Router governing infrastructure diagnostic probes.
///
/// Provisions standard stateless endpoints utilized by network proxies, load balancers,
/// and container orchestrators (such as Kubernetes pods) to evaluate host operational availability.
pub struct HealthRouter;

impl HealthRouter {
    /// Instantiates a new stateless network health diagnostic boundary layer.
    pub fn new() -> Self {
        Self {}
    }

    /// Composes and registers standard diagnostic routes into a unified sub-routing architecture branch.
    ///
    /// # Exposed Map
    /// * `GET /health`     - Standard environment availability check.
    /// * `GET /healthz`    - Legacy and cloud-native container diagnostic check.
    /// * `GET /liveness`   - Kubernetes liveness probe context (asserts container process is active).
    /// * `GET /readiness`  - Kubernetes readiness probe context (asserts network instance is ready to ingest active traffic).
    pub fn router(self) -> Router {
        Router::new()
            .route("/health", get(Self::get_ok))
            .route("/healthz", get(Self::get_ok))
            .route("/liveness", get(Self::get_ok))
            .route("/readiness", get(Self::get_ok))
    }

    /// Stateless Axum endpoint handler returning an immutable string indicator to validate thread execution.
    async fn get_ok() -> impl IntoResponse {
        "OK".into_response()
    }
}