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

//! Events, subscriptions and dead letter routes.

pub mod dlq;
pub mod events;
pub mod subscriptions;

use std::sync::Arc;

use axum::Router;
pub use dlq::DeadLetterRouter;
pub use events::EventsRouter;
pub use subscriptions::SubscriptionsRouter;
use ymir::http::OauthHttpMiddleware;
use ymir::services::token_validator::OauthTokenValidatorTrait;

use crate::routes as routes;
use crate::services::event_bus::EventBus;
use crate::services::event_sink::EventSinkTrait;

/// Events API: the feed and its live stream at the root, plus subscriptions and the DLQ.
pub struct EventsHttpRouter;

impl EventsHttpRouter {
    /// All event routes publishing directly to the local bus behind the OAuth middleware.
    pub fn build(bus: Arc<EventBus>, validator: Arc<dyn OauthTokenValidatorTrait>) -> Router {
        let sink: Arc<dyn EventSinkTrait> = bus.clone();
        Self::build_with_sink(bus, sink, validator)
    }

    /// All event routes publishing through the full configured sink pipeline.
    pub fn build_with_sink(
        bus: Arc<EventBus>,
        sink: Arc<dyn EventSinkTrait>,
        validator: Arc<dyn OauthTokenValidatorTrait>,
    ) -> Router {
        Router::new()
            .merge(EventsRouter::with_sink(bus.clone(), sink).router())
            .nest(
                routes::subscriptions::PREFIX,
                SubscriptionsRouter::new(bus.subscription_repo()).router(),
            )
            .nest(routes::dlq::PREFIX, DeadLetterRouter::new(bus).router())
            .route_layer(axum::middleware::from_fn_with_state(
                validator,
                OauthHttpMiddleware::run,
            ))
    }
}
