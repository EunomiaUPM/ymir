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

//! Event routes.

use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;
use urn::Urn;
use uuid::Uuid;
use ymir::errors::{AppResult, Errors};
use ymir::oauth::{Owner, OwnerScope};
use ymir::paginated_spec::{Cursor, Paginated};
use ymir::query::{QueryFilter, QuerySpec};
use ymir::types::oauth::UserInfo;

use crate::entities::commands::PublishEventRequest;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::envelope::{
    DeliveryClass, EventActor, EventCategory, EventEnvelope, EventEnvelopeDraft, EventOutcome,
    EventSeverity, EventSource,
};
use crate::entities::queries::EventFilter;
use crate::entities::topic::Topic;
use crate::entities::topic_pattern::TopicPattern;
use crate::routes as routes;
use crate::services::event_bus::{EventBus, EventView};
use crate::services::event_sink::EventSinkTrait;

/// Query string of the event listing.
pub type EventsQuery = QuerySpec<EventFilter>;

/// Query string of the live stream: a topic pattern and filter.
#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    #[serde(flatten)]
    pub filter: EventFilter,
}

/// How many events a read count or a mark-read touched.
#[derive(Debug, Serialize)]
pub struct EventCount {
    pub count: u64,
}

/// Shared state backing event HTTP endpoints.
#[derive(Clone)]
pub struct EventsHttpState {
    pub bus: Arc<EventBus>,
    pub sink: Arc<dyn EventSinkTrait>,
}

/// Event routes: publish, list, live stream, read state and the deliveries of each event.
#[derive(Clone)]
pub struct EventsRouter {
    state: EventsHttpState,
}

impl EventsRouter {
    /// Builds routes publishing directly to the local `EventBus`.
    pub fn new(bus: Arc<EventBus>) -> Self {
        let sink: Arc<dyn EventSinkTrait> = bus.clone();
        Self {
            state: EventsHttpState { bus, sink },
        }
    }

    /// Builds routes publishing through the full configured `sink` pipeline.
    pub fn with_sink(bus: Arc<EventBus>, sink: Arc<dyn EventSinkTrait>) -> Self {
        Self {
            state: EventsHttpState { bus, sink },
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                routes::entries::ROOT,
                get(Self::handle_list_events).post(Self::handle_publish),
            )
            .route(routes::entries::STREAM, get(Self::handle_stream))
            .route(routes::entries::UNREAD_COUNT, get(Self::handle_unread_count))
            .route(routes::entries::READ_ALL, post(Self::handle_read_all))
            .route(routes::entries::BY_ID, get(Self::handle_get_event))
            .route(routes::entries::READ, post(Self::handle_read))
            .route(routes::entries::DELIVERIES, get(Self::handle_get_deliveries))
            .with_state(self.state)
    }

    fn envelope_from_request(
        user: &UserInfo,
        req: PublishEventRequest,
    ) -> AppResult<EventEnvelope> {
        let owner = Owner::of(user, req.visibility);
        let topic = Topic::new(req.topic).map_err(|e| Errors::validation(e, None))?;
        let crate_name = req.source_crate.unwrap_or_else(|| "events".to_string());
        let schema_version = req.schema_version.unwrap_or(1);
        let payload_type = req
            .payload_type
            .unwrap_or_else(|| format!("{crate_name}.{}.v{schema_version}", topic.as_str()));

        let mut envelope = EventEnvelope::build(EventEnvelopeDraft {
            visibility_owner: owner,
            topic,
            event_type: req.event_type.unwrap_or(EventCategory::AuditLifecycle),
            severity: req.severity.unwrap_or(EventSeverity::Info),
            outcome: req.outcome.or(Some(EventOutcome::Success)),
            delivery_class: req.delivery_class.unwrap_or(DeliveryClass::AuditDurable),
            schema_version,
            payload_type,
            source: req
                .source
                .unwrap_or_else(|| EventSource::for_crate(crate_name)),
            actor: req.actor.unwrap_or_else(|| EventActor::human(user)),
            subject: req.subject,
            correlation_id: req.correlation_id,
            causation_id: req.causation_id,
            trace_context: req.trace_context,
            flow_refs: req.flow_refs.unwrap_or_default(),
            payload: req.payload,
        });
        if let Some(id) = req.id {
            envelope.id = id;
        }
        if let Some(timestamp) = req.timestamp {
            envelope.timestamp = timestamp;
        }
        if let Some(sequence) = req.sequence {
            envelope.sequence = Some(sequence);
        }
        Ok(envelope)
    }

    async fn handle_publish(
        State(state): State<EventsHttpState>,
        user: UserInfo,
        Json(req): Json<PublishEventRequest>,
    ) -> AppResult<(StatusCode, Json<EventView>)> {
        let mut envelope = Self::envelope_from_request(&user, req)?;
        if let Some(identity) = state.bus.node_identity() {
            envelope.apply_node_identity(&identity.snapshot());
        }
        state.sink.emit_envelope(envelope.clone()).await?;
        Ok((StatusCode::CREATED, Json(EventView::from_envelope(envelope))))
    }

    async fn handle_list_events(
        State(state): State<EventsHttpState>,
        user: UserInfo,
        Query(query): Query<EventsQuery>,
    ) -> AppResult<Json<Paginated<EventView>>> {
        query.filter.validate()?;
        let page = query.page.clamped();
        let (events, total) = state
            .bus
            .event_repo()
            .list_events(
                &OwnerScope::seeing(&user),
                &query.filter,
                Some(user.id().to_string()),
                &page,
                &query.sort,
            )
            .await?;
        Ok(Json(
            Paginated::from_page(events, &page, Some(total), |last| {
                Cursor::encode_composite(&last.envelope.timestamp, last.envelope.id.as_ref())
            })
            .map(EventView::assemble),
        ))
    }

    /// Server-sent events the caller sees, optionally narrowed by `topic`.
    async fn handle_stream(
        State(state): State<EventsHttpState>,
        user: UserInfo,
        Query(query): Query<StreamQuery>,
    ) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
        let scope = OwnerScope::seeing(&user);
        let pattern = query
            .filter
            .topic_pattern()?
            .unwrap_or_else(TopicPattern::match_all);
        let receiver = state.bus.subscribe();
        let events = stream::unfold(
            (receiver, pattern, scope),
            |(mut receiver, pattern, scope)| async move {
                loop {
                    match receiver.recv().await {
                        Ok(envelope)
                            if scope.admits_owner(&envelope.visibility_owner)
                                && pattern.matches(&envelope.topic) =>
                        {
                            let view = EventView::from_envelope(envelope);
                            let Ok(data) = serde_json::to_string(&view) else {
                                continue;
                            };
                            // Unnamed events, so browsers receive them through `onmessage`.
                            let event = Event::default().data(data);
                            return Some((Ok(event), (receiver, pattern, scope)));
                        }
                        Ok(_) | Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => return None,
                    }
                }
            },
        );
        Ok(Sse::new(events).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
    }

    async fn handle_get_event(
        State(state): State<EventsHttpState>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<EventView>> {
        let (urn, uuid) = Self::parse_id(&id)?;
        let stored = state
            .bus
            .event_repo()
            .get_event_by_id(&OwnerScope::seeing(&user), &urn, Some(user.id().to_string()))
            .await?
            .ok_or_else(|| Errors::missing_resource(uuid.to_string(), "event not found", None))?;
        Ok(Json(EventView::assemble(stored)))
    }

    async fn handle_unread_count(
        State(state): State<EventsHttpState>,
        user: UserInfo,
    ) -> AppResult<Json<EventCount>> {
        let count = state
            .bus
            .event_repo()
            .count_unread(&OwnerScope::seeing(&user), Some(user.id().to_string()))
            .await?;
        Ok(Json(EventCount { count }))
    }

    /// Marks every unread event the caller sees as read.
    async fn handle_read_all(
        State(state): State<EventsHttpState>,
        user: UserInfo,
    ) -> AppResult<Json<EventCount>> {
        let count = state
            .bus
            .event_repo()
            .mark_read(&OwnerScope::seeing(&user), Some(user.id().to_string()), None)
            .await?;
        Ok(Json(EventCount { count }))
    }

    async fn handle_read(
        State(state): State<EventsHttpState>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<EventCount>> {
        let (urn, _) = Self::parse_id(&id)?;
        let count = state
            .bus
            .event_repo()
            .mark_read(&OwnerScope::seeing(&user), Some(user.id().to_string()), Some(urn))
            .await?;
        Ok(Json(EventCount { count }))
    }

    async fn handle_get_deliveries(
        State(state): State<EventsHttpState>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<Vec<EventDeliveryRecord>>> {
        let deliveries = state
            .bus
            .delivery_repo()
            .list_by_event(&OwnerScope::seeing(&user), &id)
            .await?;
        Ok(Json(deliveries))
    }

    // Validate and parse an ID string into a canonical URN and UUID.
    fn parse_id(id: &str) -> AppResult<(Urn, Uuid)> {
        if let Some(uuid_str) = id.strip_prefix("urn:uuid:") {
            let u =
                Uuid::parse_str(uuid_str).map_err(|e| Errors::validation(e.to_string(), None))?;
            let urn = Urn::from_str(id).map_err(|e| Errors::validation(e.to_string(), None))?;
            Ok((urn, u))
        } else if let Ok(u) = Uuid::parse_str(id) {
            let urn = Urn::from_str(&format!("urn:uuid:{u}"))
                .map_err(|e| Errors::validation(e.to_string(), None))?;
            Ok((urn, u))
        } else {
            Err(Errors::validation(
                "id must be a valid UUID or urn:uuid:<uuid>",
                None,
            ))
        }
    }
}
