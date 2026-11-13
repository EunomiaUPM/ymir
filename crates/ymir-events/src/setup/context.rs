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

use std::sync::{Arc, Mutex};

use ymir::boot::workers::BackgroundWorker;
use ymir::errors::Outcome;
use ymir::module_loader::root_context::RootContext;

use crate::data::factory::DataFactory;
use crate::data::sea_orm::SeaOrmDataFactory;
use crate::services::event_bus::policy::RetryPolicy;
use crate::services::event_bus::worker::RetryWorker;
use crate::services::event_bus::EventBus;
use crate::services::event_sink::{EventPipelineBuilder, EventSinkTrait};

/// The local event bus, unified multi-sink pipeline, and background delivery workers.
#[derive(Clone)]
pub struct AppContext {
    pub event_bus: Arc<EventBus>,
    pub event_sink: Arc<dyn EventSinkTrait>,
    pub retry_worker: RetryWorker,
    extra_workers: Arc<Mutex<Vec<Box<dyn BackgroundWorker>>>>,
}

impl AppContext {
    /// Builds the event bus, webhook retry worker, and multi-sink pipeline from `root`.
    pub fn try_build(root: &RootContext, policy: Option<RetryPolicy>) -> Outcome<Self> {
        let policy = policy.unwrap_or_default();
        let factory = SeaOrmDataFactory::new(root.db.clone());

        let event_repo = factory.event_repository();
        let subscription_repo = factory.subscription_repository();
        let delivery_repo = factory.delivery_repository();
        let dlq_repo = factory.dlq_repository();
        let outbox_repo = factory.outbox_repository();

        let event_bus = Arc::new(
            EventBus::new(
                event_repo.clone(),
                subscription_repo.clone(),
                delivery_repo.clone(),
                dlq_repo.clone(),
                policy.clone(),
                1024,
            )
            .with_node_identity(root.node_identity.clone()),
        );

        let local_bus_sink: Arc<dyn EventSinkTrait> = event_bus.clone();
        let pipeline = EventPipelineBuilder::build(
            &root.events_config,
            root.node_identity.clone(),
            Some(local_bus_sink),
            Some(outbox_repo),
        )?;

        let retry_worker = RetryWorker::new(
            event_repo,
            subscription_repo,
            delivery_repo,
            dlq_repo,
            event_bus.dispatcher(),
            policy,
        );

        Ok(Self {
            event_bus,
            event_sink: pipeline.sink,
            retry_worker,
            extra_workers: Arc::new(Mutex::new(pipeline.workers)),
        })
    }

    /// Panicking convenience wrapper when called after [`RootContext`] config validation.
    pub fn build(root: &RootContext, policy: Option<RetryPolicy>) -> Self {
        Self::try_build(root, policy).expect("valid events pipeline configuration")
    }

    /// Drains auxiliary sink workers (such as the Kafka outbox relay) once on boot.
    pub fn take_extra_workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        let Ok(mut guard) = self.extra_workers.lock() else {
            return Vec::new();
        };
        std::mem::take(&mut *guard)
    }
}
