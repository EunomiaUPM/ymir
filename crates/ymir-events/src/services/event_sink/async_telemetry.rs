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

//! Non-blocking telemetry sink buffering `DeliveryClass::TelemetryStream` events off request paths.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::warn;
use ymir::boot::workers::BackgroundWorker;
use ymir::errors::Outcome;

use crate::entities::envelope::{DeliveryClass, EventEnvelope};
use crate::services::event_sink::EventSinkTrait;

/// Forwards durable audit events synchronously while queuing telemetry streams in memory.
pub struct AsyncTelemetrySink {
    inner: Arc<dyn EventSinkTrait>,
    tx: mpsc::Sender<EventEnvelope>,
    dropped: AtomicU64,
}

impl AsyncTelemetrySink {
    /// Default queue capacity and per-flush batch size for background telemetry persistence.
    pub const DEFAULT_QUEUE_CAPACITY: usize = 4096;
    pub const DEFAULT_BATCH_SIZE: usize = 128;

    /// Creates a sink and its paired background flush worker.
    pub fn open(inner: Arc<dyn EventSinkTrait>, capacity: usize) -> (Self, TelemetryFlushWorker) {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        let sink = Self {
            inner: inner.clone(),
            tx,
            dropped: AtomicU64::new(0),
        };
        let worker = TelemetryFlushWorker::new(inner, rx, Self::DEFAULT_BATCH_SIZE);
        (sink, worker)
    }

    fn enqueue_telemetry(&self, envelope: EventEnvelope) {
        if self.tx.try_send(envelope).is_err() {
            let count = self.dropped.fetch_add(1, Ordering::Relaxed) + 1;
            if count == 1 || count.is_multiple_of(256) {
                warn!(dropped = count, "Telemetry event queue full or closed; dropping event");
            }
        }
    }
}

#[async_trait]
impl EventSinkTrait for AsyncTelemetrySink {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        match envelope.delivery_class {
            DeliveryClass::TelemetryStream => {
                self.enqueue_telemetry(envelope);
                Ok(())
            }
            DeliveryClass::AuditDurable => self.inner.emit_envelope(envelope).await,
        }
    }

    async fn emit_envelopes(&self, envelopes: Vec<EventEnvelope>) -> Outcome<()> {
        let mut audit = Vec::new();
        for envelope in envelopes {
            match envelope.delivery_class {
                DeliveryClass::TelemetryStream => self.enqueue_telemetry(envelope),
                DeliveryClass::AuditDurable => audit.push(envelope),
            }
        }
        if audit.is_empty() {
            Ok(())
        } else {
            self.inner.emit_envelopes(audit).await
        }
    }
}

/// Drains queued telemetry envelopes in batches to the underlying sink pipeline.
pub struct TelemetryFlushWorker {
    inner: Arc<dyn EventSinkTrait>,
    rx: mpsc::Receiver<EventEnvelope>,
    batch_size: usize,
}

impl TelemetryFlushWorker {
    pub fn new(
        inner: Arc<dyn EventSinkTrait>,
        rx: mpsc::Receiver<EventEnvelope>,
        batch_size: usize,
    ) -> Self {
        Self {
            inner,
            rx,
            batch_size: batch_size.max(1),
        }
    }

    async fn flush_batch(&mut self, first: EventEnvelope) {
        let mut batch = Vec::with_capacity(self.batch_size);
        batch.push(first);
        while batch.len() < self.batch_size {
            match self.rx.try_recv() {
                Ok(next) => batch.push(next),
                Err(_) => break,
            }
        }
        if let Err(err) = self.inner.emit_envelopes(batch).await {
            warn!(error = ?err, "Failed to flush telemetry event batch");
        }
    }

    async fn drain_remaining(&mut self) {
        while let Ok(first) = self.rx.try_recv() {
            self.flush_batch(first).await;
        }
    }
}

#[async_trait]
impl BackgroundWorker for TelemetryFlushWorker {
    fn name(&self) -> &'static str {
        "telemetry-event-flush"
    }

    async fn run(mut self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    self.drain_remaining().await;
                    return Ok(());
                }
                maybe = self.rx.recv() => match maybe {
                    Some(first) => self.flush_batch(first).await,
                    None => return Ok(()),
                }
            }
        }
    }
}
