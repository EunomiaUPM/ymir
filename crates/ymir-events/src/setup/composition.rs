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

use std::sync::Arc;

use axum::Router;
use sea_orm_migration::MigrationTrait;
use ymir::boot::workers::BackgroundWorker;
use ymir::errors::Outcome;
use ymir::module_loader::root_context::RootContext;
use ymir::module_loader::service_module::ServiceModuleTrait;
use ymir::services::token_validator::OauthTokenValidatorTrait;

use crate::http::EventsHttpRouter;
use crate::services::event_bus::EventBus;
use crate::services::event_sink::{BuiltEventPipeline, EventPipelineBuilder, EventSinkTrait};
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;

/// Events service module integrating migrations and HTTP routes into the modular host.
pub struct EventsModule {
    ctx: Arc<AppContext>,
    validator: Arc<dyn OauthTokenValidatorTrait>,
}

impl EventsModule {
    /// Builds the bus, sink pipeline and retry worker, returning validation errors explicitly.
    pub fn try_compose(root: &RootContext) -> Outcome<Self> {
        Ok(Self {
            ctx: Arc::new(AppContext::try_build(root, None)?),
            validator: root.validator.clone(),
        })
    }

    /// Builds the bus on the shared database, with the default retry policy.
    pub fn compose(root: &RootContext) -> Self {
        Self::try_compose(root).expect("valid events pipeline configuration")
    }

    /// Local bus handle (in-process SSE stream and webhook dispatcher).
    pub fn event_bus(&self) -> EventBus {
        (*self.ctx.event_bus).clone()
    }

    /// Canonical multi-sink handle handed out to every domain module.
    pub fn event_sink(&self) -> Arc<dyn EventSinkTrait> {
        self.ctx.event_sink.clone()
    }

    /// Builds a standalone multi-sink pipeline for worker binaries without an `EventsModule`.
    pub fn build_standalone_pipeline(
        root: &RootContext,
        local_bus: Option<Arc<dyn EventSinkTrait>>,
    ) -> Outcome<BuiltEventPipeline> {
        EventPipelineBuilder::build(
            &root.events_config,
            root.node_identity.clone(),
            local_bus,
            None,
        )
    }

    /// Return all SeaORM database migrations for the events bus, read states and outbox.
    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::data::migrations::get_events_migrations()
    }
}

impl ServiceModuleTrait for EventsModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn http(&self) -> Option<(String, Router)> {
        let router = EventsHttpRouter::build_with_sink(
            self.ctx.event_bus.clone(),
            self.ctx.event_sink.clone(),
            self.validator.clone(),
        );
        Some((format!("/api/v1/{SERVICE_NAME}"), router))
    }

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        let mut workers: Vec<Box<dyn BackgroundWorker>> =
            vec![Box::new(self.ctx.retry_worker.clone())];
        workers.extend(self.ctx.take_extra_workers());
        workers
    }
}

/// Attaches standalone event-pipeline background workers to an agent without `EventsModule`.
pub struct StandaloneEventsModule {
    workers: std::sync::Mutex<Vec<Box<dyn BackgroundWorker>>>,
}

impl StandaloneEventsModule {
    /// Creates a module carrying standalone pipeline workers (e.g., Kafka outbox relay).
    pub fn new(workers: Vec<Box<dyn BackgroundWorker>>) -> Self {
        Self {
            workers: std::sync::Mutex::new(workers),
        }
    }
}

impl ServiceModuleTrait for StandaloneEventsModule {
    fn name(&self) -> &'static str {
        "events_pipeline"
    }

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        std::mem::take(
            &mut *self
                .workers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }
}
