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

//! Multi-sink event pipeline: normalizes node identity and fans out across configured sinks.

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use opentelemetry::metrics::Counter;
use opentelemetry::{KeyValue, global};
use tracing::warn;
use ymir::boot::workers::BackgroundWorker;
use ymir::config::types::{EventNodeIdentity, EventsConfig, KafkaFailureMode};
use ymir::errors::{Errors, Outcome};

use crate::data::repo::EventOutboxRepo;
use crate::entities::envelope::EventEnvelope;
use crate::services::event_sink::noop::NoopEventSink;
use crate::services::event_sink::otlp::OtlpEventSink;
use crate::services::event_sink::remote_bus::RemoteBusSink;
use crate::services::event_sink::stdout_json::TracingJsonSink;
use crate::services::event_sink::EventSinkTrait;

#[cfg(feature = "kafka")]
use crate::services::event_sink::kafka::{KafkaEventSink, KafkaOutboxWorker};

static SINK_FAILURES: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter("events")
        .u64_counter("events.sink.failures")
        .build()
});

#[derive(Clone)]
struct RegisteredSink {
    name: &'static str,
    critical: bool,
    sink: Arc<dyn EventSinkTrait>,
}

/// Stamps node identity once on every envelope and fans out to configured sinks.
#[derive(Clone)]
pub struct CompositeEventSink {
    node_identity: Arc<EventNodeIdentity>,
    sinks: Vec<RegisteredSink>,
}

impl CompositeEventSink {
    /// Creates an empty pipeline bound to the process [`EventNodeIdentity`].
    pub fn new(node_identity: Arc<EventNodeIdentity>) -> Self {
        Self {
            node_identity,
            sinks: Vec::new(),
        }
    }

    /// Registers a required local durability sink whose failure propagates to the caller.
    pub fn add_critical_sink(&mut self, name: &'static str, sink: Arc<dyn EventSinkTrait>) {
        self.sinks.push(RegisteredSink {
            name,
            critical: true,
            sink,
        });
    }

    /// Registers an auxiliary observability or broker sink (failures are logged and metered).
    pub fn add_sink(&mut self, name: &'static str, sink: Arc<dyn EventSinkTrait>) {
        self.sinks.push(RegisteredSink {
            name,
            critical: false,
            sink,
        });
    }

    /// Returns registered sink names in execution order (diagnostics and tests).
    pub fn sink_names(&self) -> Vec<&'static str> {
        self.sinks.iter().map(|entry| entry.name).collect()
    }
}

#[async_trait]
impl EventSinkTrait for CompositeEventSink {
    async fn emit_envelope(&self, mut envelope: EventEnvelope) -> Outcome<()> {
        let snapshot = self.node_identity.snapshot();
        envelope.apply_node_identity(&snapshot);

        let mut first_critical_err: Option<Errors> = None;
        for entry in &self.sinks {
            if let Err(err) = entry.sink.emit_envelope(envelope.clone()).await {
                SINK_FAILURES.add(1, &[KeyValue::new("sink", entry.name)]);
                if entry.critical {
                    if first_critical_err.is_none() {
                        first_critical_err = Some(err);
                    }
                } else {
                    warn!(
                        sink = entry.name,
                        event_id = %envelope.id,
                        error = ?err,
                        "Non-critical event sink failed"
                    );
                }
            }
        }

        match first_critical_err {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

/// Composed sink plus optional background workers (e.g. Kafka outbox relay).
pub struct BuiltEventPipeline {
    pub sink: Arc<dyn EventSinkTrait>,
    pub workers: Vec<Box<dyn BackgroundWorker>>,
}

/// Constructs [`CompositeEventSink`] strictly from [`EventsConfig`] and runtime dependencies.
pub struct EventPipelineBuilder;

impl EventPipelineBuilder {
    /// Builds a standalone pipeline directly from `RootContext` (for binaries without `EventsModule`).
    pub fn from_root(
        root: &ymir::module_loader::root_context::RootContext,
        local_bus: Option<Arc<dyn EventSinkTrait>>,
    ) -> Outcome<BuiltEventPipeline> {
        Self::build(
            &root.events_config,
            root.node_identity.clone(),
            local_bus,
            None,
        )
    }

    /// Builds the configured sink pipeline without failing when optional exporters are off.
    pub fn build(
        config: &EventsConfig,
        node_identity: Arc<EventNodeIdentity>,
        local_bus: Option<Arc<dyn EventSinkTrait>>,
        outbox_repo: Option<Arc<dyn EventOutboxRepo>>,
    ) -> Outcome<BuiltEventPipeline> {
        config.validate()?;

        let mut composite = CompositeEventSink::new(node_identity);
        let mut workers: Vec<Box<dyn BackgroundWorker>> = Vec::new();

        if config.local_store.should_enable(local_bus.is_some()) {
            if let Some(bus) = local_bus {
                composite.add_critical_sink("local_store", bus);
            }
        }

        if config.otlp.enabled {
            if let Some(otlp_sink) = OtlpEventSink::from_global(&config.otlp) {
                composite.add_sink("otlp", Arc::new(otlp_sink));
            }
        }

        if config.stdout_json.enabled {
            composite.add_sink(
                "stdout_json",
                Arc::new(TracingJsonSink::new(config.stdout_json.clone())),
            );
        }

        if config.remote_bus.enabled {
            let remote = RemoteBusSink::new(config.remote_bus.clone())?;
            composite.add_sink("remote_bus", Arc::new(remote));
        }

        if config.kafka.enabled {
            Self::attach_kafka(config, &mut composite, &mut workers, outbox_repo)?;
        }

        let sink: Arc<dyn EventSinkTrait> = if composite.sinks.is_empty() {
            Arc::new(NoopEventSink)
        } else {
            Arc::new(composite)
        };

        Ok(BuiltEventPipeline { sink, workers })
    }

    #[cfg(feature = "kafka")]
    fn attach_kafka(
        config: &EventsConfig,
        composite: &mut CompositeEventSink,
        workers: &mut Vec<Box<dyn BackgroundWorker>>,
        outbox_repo: Option<Arc<dyn EventOutboxRepo>>,
    ) -> Outcome<()> {
        let kafka_sink = KafkaEventSink::new(config.kafka.clone(), outbox_repo.clone())?;
        composite.add_sink("kafka", Arc::new(kafka_sink));
        if config.kafka.failure_mode == KafkaFailureMode::OutboxRelay {
            if let Some(repo) = outbox_repo {
                workers.push(Box::new(KafkaOutboxWorker::new(&config.kafka, repo)?));
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "kafka"))]
    fn attach_kafka(
        _: &EventsConfig,
        _: &mut CompositeEventSink,
        _: &mut Vec<Box<dyn BackgroundWorker>>,
        _: Option<Arc<dyn EventOutboxRepo>>,
    ) -> Outcome<()> {
        Err(Errors::validation(
            "events.kafka.enabled is true, but ymir-events was compiled without the 'kafka' feature",
            None,
        ))
    }
}
