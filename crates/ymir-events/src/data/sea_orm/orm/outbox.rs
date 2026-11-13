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

//! `event_outbox` table storing pending broker deliveries for `OutboxRelay`.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;

use crate::entities::outbox::{OutboxRecord, OutboxStatus};

/// `event_outbox` row.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_outbox")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub sink_name: String,
    pub event_id: String,
    pub destination: String,
    pub partition_key: String,
    pub envelope_json: serde_json::Value,
    pub attempts: i32,
    pub next_retry_at: chrono::NaiveDateTime,
    pub last_error: Option<String>,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub delivered_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Converts a stored outbox row into its domain record.
    pub fn into_domain(self) -> OutboxRecord {
        OutboxRecord {
            id: self.id,
            sink_name: self.sink_name,
            event_id: self.event_id,
            destination: self.destination,
            partition_key: self.partition_key,
            envelope_json: self.envelope_json,
            attempts: self.attempts.max(0) as u32,
            next_retry_at: DateTime::from_naive_utc_and_offset(self.next_retry_at, Utc),
            last_error: self.last_error,
            status: OutboxStatus::from_str(&self.status).unwrap_or(OutboxStatus::Pending),
            created_at: DateTime::from_naive_utc_and_offset(self.created_at, Utc),
            delivered_at: self
                .delivered_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }
    }
}

impl ActiveModel {
    /// Converts a domain outbox record into an insertable SeaORM active model.
    pub fn from_domain(record: &OutboxRecord) -> Self {
        Self {
            id: ActiveValue::Set(record.id.clone()),
            sink_name: ActiveValue::Set(record.sink_name.clone()),
            event_id: ActiveValue::Set(record.event_id.clone()),
            destination: ActiveValue::Set(record.destination.clone()),
            partition_key: ActiveValue::Set(record.partition_key.clone()),
            envelope_json: ActiveValue::Set(record.envelope_json.clone()),
            attempts: ActiveValue::Set(record.attempts as i32),
            next_retry_at: ActiveValue::Set(record.next_retry_at.naive_utc()),
            last_error: ActiveValue::Set(record.last_error.clone()),
            status: ActiveValue::Set(record.status.as_str().to_string()),
            created_at: ActiveValue::Set(record.created_at.naive_utc()),
            delivered_at: ActiveValue::Set(record.delivered_at.map(|dt| dt.naive_utc())),
        }
    }
}
