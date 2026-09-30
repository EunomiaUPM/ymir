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

//! OTel instruments of the outbound client, bound to the process meter provider.

use std::sync::LazyLock;

use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{KeyValue, global};

pub(crate) struct ClientMetrics {
    pub duration: Histogram<f64>,
    pub retries: Counter<u64>,
    pub circuit_transitions: Counter<u64>,
}

/// Resolved on the first outbound call, which follows the process telemetry set-up.
pub(crate) static METRICS: LazyLock<ClientMetrics> = LazyLock::new(|| {
    let meter = global::meter("ymir.client");
    ClientMetrics {
        duration: meter
            .f64_histogram("http.client.request.duration")
            .with_unit("s")
            .build(),
        retries: meter.u64_counter("http.client.retries").build(),
        circuit_transitions: meter.u64_counter("http.client.circuit.transitions").build(),
    }
});

impl ClientMetrics {
    pub fn circuit(&self, host: &str, state: &'static str) {
        self.circuit_transitions.add(
            1,
            &[
                KeyValue::new("server.address", host.to_string()),
                KeyValue::new("circuit.state", state),
            ],
        );
    }
}
