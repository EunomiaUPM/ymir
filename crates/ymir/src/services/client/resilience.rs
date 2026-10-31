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

//! Per-host protection of outbound calls: a bulkhead so one slow host cannot starve the
//! others, a circuit breaker so a dead host fails fast, and jittered retry backoff.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rand::Rng;
use tokio::sync::Semaphore;

use crate::services::client::metrics::METRICS;

/// When a host's breaker opens and how long it stays open before one probe is let through.
#[derive(Clone, Copy, Debug)]
pub struct BreakerConfig {
    pub failure_threshold: u32,
    pub open_for: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl BreakerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::HalfOpen => "half_open",
        }
    }
}

#[derive(Debug)]
enum Circuit {
    Closed {
        failures: u32,
    },
    Open {
        until: Instant,
    },
    /// A probe older than `open_for` is presumed lost (e.g. cancelled) and replaced.
    HalfOpen {
        probe_started: Instant,
    },
}

/// Consecutive-failure breaker: network errors, timeouts and `5xx` count; any other answer
/// proves the host alive and closes it.
#[derive(Debug)]
pub struct CircuitBreaker {
    host: String,
    config: BreakerConfig,
    circuit: Mutex<Circuit>,
}

impl CircuitBreaker {
    pub fn new(host: &str, config: BreakerConfig) -> Self {
        Self {
            host: host.to_string(),
            config,
            circuit: Mutex::new(Circuit::Closed { failures: 0 }),
        }
    }

    fn transition(&self, state: BreakerState) {
        METRICS.circuit(&self.host, state.as_str());
    }

    /// Whether a call may go out now; an elapsed open window admits a single probe.
    pub fn try_acquire(&self) -> bool {
        let mut circuit = self.lock();
        let now = Instant::now();
        match *circuit {
            Circuit::Closed { .. } => true,
            Circuit::Open { until } if now >= until => {
                *circuit = Circuit::HalfOpen { probe_started: now };
                tracing::info!(host = %self.host, "circuit half-open, probing");
                self.transition(BreakerState::HalfOpen);
                true
            }
            Circuit::Open { .. } => false,
            Circuit::HalfOpen { probe_started } => {
                let stale = now.duration_since(probe_started) >= self.config.open_for;
                if stale {
                    *circuit = Circuit::HalfOpen { probe_started: now };
                }
                stale
            }
        }
    }

    pub fn on_success(&self) {
        let mut circuit = self.lock();
        if !matches!(*circuit, Circuit::Closed { .. }) {
            tracing::info!(host = %self.host, "circuit closed");
            self.transition(BreakerState::Closed);
        }
        *circuit = Circuit::Closed { failures: 0 };
    }

    pub fn on_failure(&self) {
        let mut circuit = self.lock();
        let failures = match *circuit {
            Circuit::Closed { failures } => failures + 1,
            _ => self.config.failure_threshold,
        };
        *circuit = if failures >= self.config.failure_threshold {
            tracing::warn!(host = %self.host, failures, "circuit open");
            self.transition(BreakerState::Open);
            Circuit::Open {
                until: Instant::now() + self.config.open_for,
            }
        } else {
            Circuit::Closed { failures }
        };
    }

    pub fn state(&self) -> BreakerState {
        match *self.lock() {
            Circuit::Closed { .. } => BreakerState::Closed,
            Circuit::Open { until } if Instant::now() >= until => BreakerState::HalfOpen,
            Circuit::Open { .. } => BreakerState::Open,
            Circuit::HalfOpen { .. } => BreakerState::HalfOpen,
        }
    }

    /// A poisoned lock only means a panic mid-update; the state itself is still usable.
    fn lock(&self) -> std::sync::MutexGuard<'_, Circuit> {
        self.circuit.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Protection of one host: its share of in-flight requests and, optionally, its breaker.
#[derive(Debug)]
pub struct HostGuard {
    pub permits: Semaphore,
    pub breaker: Option<CircuitBreaker>,
}

/// Lazily created [`HostGuard`]s keyed by `scheme://host:port`.
#[derive(Debug)]
pub struct HostGuards {
    per_host: usize,
    breaker: Option<BreakerConfig>,
    hosts: Mutex<HashMap<String, Arc<HostGuard>>>,
}

impl HostGuards {
    pub fn new(per_host: usize, breaker: Option<BreakerConfig>) -> Self {
        Self {
            per_host,
            breaker,
            hosts: Mutex::new(HashMap::new()),
        }
    }

    pub fn guard(&self, authority: &str) -> Arc<HostGuard> {
        let mut hosts = self.hosts.lock().unwrap_or_else(|e| e.into_inner());
        hosts
            .entry(authority.to_string())
            .or_insert_with(|| {
                Arc::new(HostGuard {
                    permits: Semaphore::new(self.per_host),
                    breaker: self.breaker.map(|c| CircuitBreaker::new(authority, c)),
                })
            })
            .clone()
    }

    /// Hosts whose breaker is not closed, for health reporting.
    pub fn degraded(&self) -> Vec<(String, BreakerState)> {
        let hosts = self.hosts.lock().unwrap_or_else(|e| e.into_inner());
        hosts
            .iter()
            .filter_map(|(host, guard)| {
                let state = guard.breaker.as_ref()?.state();
                (state != BreakerState::Closed).then(|| (host.clone(), state))
            })
            .collect()
    }
}

/// Exponential backoff with full jitter: uniform in `[0, min(cap, base * 2^(attempt-1))]`.
#[derive(Clone, Copy, Debug)]
pub struct Backoff {
    pub base: Duration,
    pub cap: Duration,
}

impl Backoff {
    pub fn delay(&self, attempt: u32) -> Duration {
        let exp = self
            .base
            .saturating_mul(1u32 << attempt.saturating_sub(1).min(16));
        let ceiling = exp.min(self.cap);
        let millis = ceiling.as_millis() as u64;
        Duration::from_millis(rand::thread_rng().gen_range(0..=millis))
    }
}
