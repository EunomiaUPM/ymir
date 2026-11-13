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

//! Webhook subscriptions and payload delivery formats.

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ymir::oauth::Owner;
use ymir::secret::Secret;

use crate::entities::topic::Topic;
use crate::entities::topic_pattern::TopicPattern;

/// Controls the JSON body posted to a webhook endpoint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookDeliveryFormat {
    /// Posts the full immutable canonical `EventEnvelope` v2 JSON (recommended default).
    #[serde(alias = "Envelope")]
    #[default]
    Envelope,
    /// Posts only `envelope.payload` for backward compatibility with legacy consumers.
    #[serde(alias = "PayloadOnly")]
    PayloadOnly,
}

impl WebhookDeliveryFormat {
    /// Stable lowercase database and config identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Envelope => "envelope",
            Self::PayloadOnly => "payload_only",
        }
    }
}

impl Display for WebhookDeliveryFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for WebhookDeliveryFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "envelope" => Ok(Self::Envelope),
            "payload_only" | "payloadonly" | "payload-only" => Ok(Self::PayloadOnly),
            other => Err(format!("unknown webhook delivery format '{other}'")),
        }
    }
}

/// Webhook subscription: where to deliver, which topics, format, signing secret and retries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRecord {
    pub id: String,
    /// Who the record belongs to: the subscriber.
    #[serde(flatten)]
    pub owner: Owner,
    pub callback_address: String,
    pub topic_pattern: TopicPattern,
    pub secret: Option<Secret<String>>,
    pub headers: Option<HashMap<String, String>>,
    pub retry_limit: Option<u32>,
    #[serde(default)]
    pub delivery_format: WebhookDeliveryFormat,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub expiration_time: Option<DateTime<Utc>>,
}

impl SubscriptionRecord {
    /// Active and not past its expiration time.
    pub fn is_active(&self) -> bool {
        if !self.active {
            return false;
        }
        if let Some(exp) = self.expiration_time {
            if exp <= Utc::now() {
                return false;
            }
        }
        true
    }

    /// Active and its pattern matches `topic`.
    pub fn matches(&self, topic: &Topic) -> bool {
        self.is_active() && self.topic_pattern.matches(topic)
    }
}
