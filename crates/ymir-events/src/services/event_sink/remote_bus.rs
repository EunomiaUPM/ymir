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

//! HTTP relay forwarding canonical events from standalone workers to a central event bus.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, HeaderValue};
use ymir::config::types::RemoteBusEventsConfig;
use ymir::errors::{Errors, Outcome};
use ymir::services::client::{ClientExt, ClientService};
use ymir::types::http::HttpBody;

use crate::entities::commands::PublishEventRequest;
use crate::entities::envelope::EventEnvelope;
use crate::services::event_sink::EventSinkTrait;

/// Forwards canonical envelopes over HTTP to another node's `ymir-events` ingest API.
#[derive(Clone)]
pub struct RemoteBusSink {
    client: Arc<ClientService>,
    endpoint: String,
    bearer_token: Option<String>,
}

impl RemoteBusSink {
    /// Builds a validated remote bus HTTP relay from configuration.
    pub fn new(config: RemoteBusEventsConfig) -> Outcome<Self> {
        config.validate()?;
        let endpoint = config
            .endpoint
            .as_deref()
            .unwrap_or_default()
            .trim()
            .trim_end_matches('/')
            .to_string();
        let timeout = Duration::from_millis(config.timeout_ms.max(100));
        let client = Arc::new(ClientService::builder().timeout(Some(timeout)).build());
        let bearer_token = config.resolve_bearer_token();
        Ok(Self {
            client,
            endpoint,
            bearer_token,
        })
    }

    fn build_headers(&self) -> Outcome<Option<HeaderMap>> {
        let Some(token) = &self.bearer_token else {
            return Ok(None);
        };
        let mut headers = HeaderMap::new();
        let value = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|e| Errors::validation(format!("invalid remote bus bearer token: {e}"), None))?;
        headers.insert(AUTHORIZATION, value);
        Ok(Some(headers))
    }
}

#[async_trait]
impl EventSinkTrait for RemoteBusSink {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> Outcome<()> {
        let req = PublishEventRequest::from_envelope(&envelope);
        let headers = self.build_headers()?;
        let body = HttpBody::json(&req)?;
        self.client.post_ok(&self.endpoint, headers, body).await
    }
}
