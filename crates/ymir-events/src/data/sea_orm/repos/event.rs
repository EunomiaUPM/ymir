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

//! SeaORM event store repository with separated `event_read_states` UI projection.

use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::sea_query::{BinOper, Expr, OnConflict, Query, SimpleExpr};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::oauth::OwnerScope;
use ymir::paginated_spec::{Cursor, Page, Sort};

use crate::data::repo::event::{EventStoreRepo, StoredEvent};
use crate::data::sea_orm::orm::{event, read_state};
use crate::data::sea_orm::repos::listing::NaiveKeyset;
use crate::entities::envelope::EventEnvelope;
use crate::entities::queries::EventFilter;
use crate::entities::topic_pattern::TopicPattern;

#[derive(Clone)]
pub struct SeaOrmEventRepo {
    db: DatabaseConnection,
}

impl SeaOrmEventRepo {
    const SYSTEM_ROOT_READER: &'static str = "__system_root__";

    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn resolve_reader(scope: &OwnerScope, reader_id: Option<String>) -> String {
        if let Some(explicit) = reader_id.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
            return explicit;
        }
        match scope {
            OwnerScope::All => Self::SYSTEM_ROOT_READER.to_string(),
            OwnerScope::OwnedBy(user_id)
            | OwnerScope::Acting { user_id, .. }
            | OwnerScope::Seeing { user_id, .. } => user_id.clone(),
        }
    }

    fn scope_condition(scope: &OwnerScope) -> sea_orm::Condition {
        scope.condition(
            event::Column::UserId,
            event::Column::UserRole,
            event::Column::Visibility,
        )
    }

    fn read_exists_expr(reader: &str) -> SimpleExpr {
        Expr::exists(
            Query::select()
                .expr(Expr::val(1))
                .from(read_state::Entity)
                .and_where(
                    Expr::col((read_state::Entity, read_state::Column::EventId))
                        .equals((event::Entity, event::Column::Id)),
                )
                .and_where(
                    Expr::col((read_state::Entity, read_state::Column::ReaderId)).eq(reader),
                )
                .take(),
        )
    }

    fn apply_scalar_filters(
        mut query: Select<event::Entity>,
        filter: &EventFilter,
        reader: &str,
    ) -> Select<event::Entity> {
        let eq_filters = [
            (event::Column::SourceCrate, &filter.source_crate),
            (event::Column::DataspaceId, &filter.dataspace_id),
            (event::Column::ParticipantId, &filter.participant_id),
            (event::Column::Service, &filter.service),
            (event::Column::PayloadType, &filter.payload_type),
            (event::Column::ActorId, &filter.actor_id),
            (event::Column::SubjectKind, &filter.subject_kind),
            (event::Column::SubjectId, &filter.subject_id),
            (event::Column::CorrelationId, &filter.correlation_id),
            (event::Column::AgreementId, &filter.agreement_id),
            (event::Column::TransferId, &filter.transfer_id),
            (event::Column::NegotiationId, &filter.negotiation_id),
            (event::Column::GrantId, &filter.grant_id),
            (event::Column::DatasetId, &filter.dataset_id),
            (event::Column::PeerId, &filter.peer_id),
        ];
        for (col, val) in eq_filters {
            if let Some(value) = val {
                query = query.filter(col.eq(value.clone()));
            }
        }
        if let Some(category) = filter.event_type {
            query = query.filter(event::Column::EventType.eq(category.as_str()));
        }
        if let Some(severity) = filter.severity {
            query = query.filter(event::Column::Severity.eq(severity.as_str()));
        }
        if let Some(outcome) = filter.outcome {
            query = query.filter(event::Column::Outcome.eq(outcome.as_str()));
        }
        match filter.read {
            Some(true) => query = query.filter(Self::read_exists_expr(reader)),
            Some(false) => query = query.filter(Self::read_exists_expr(reader).not()),
            None => {}
        }
        if let Some(after) = filter.created_after {
            query = query.filter(event::Column::Timestamp.gte(after.naive_utc()));
        }
        if let Some(before) = filter.created_before {
            query = query.filter(event::Column::Timestamp.lt(before.naive_utc()));
        }
        query
    }

    async fn fetch_read_map(
        &self,
        reader: &str,
        event_ids: &[String],
    ) -> Outcome<HashMap<String, DateTime<Utc>>> {
        if event_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows = read_state::Entity::find()
            .filter(read_state::Column::ReaderId.eq(reader))
            .filter(read_state::Column::EventId.is_in(event_ids.to_vec()))
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to fetch read states", Some(Box::new(e))))?;
        Ok(rows
            .into_iter()
            .map(|r| (r.event_id, DateTime::from_naive_utc_and_offset(r.read_at, Utc)))
            .collect())
    }

    async fn attach_read_states(
        &self,
        models: Vec<event::Model>,
        reader: &str,
    ) -> Outcome<Vec<StoredEvent>> {
        let ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
        let read_map = self.fetch_read_map(reader, &ids).await?;
        models
            .into_iter()
            .map(|m| {
                let id_str = m.id.clone();
                let envelope = m.into_domain()?;
                Ok(StoredEvent {
                    read_at: read_map.get(&id_str).copied(),
                    envelope,
                })
            })
            .collect()
    }

    async fn list_sqlite_with_topic(
        &self,
        query: Select<event::Entity>,
        pattern: &TopicPattern,
        reader: &str,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<StoredEvent>, u64)> {
        let ordered = if sort.is_ascending() {
            query
                .order_by_asc(event::Column::Timestamp)
                .order_by_asc(event::Column::Id)
        } else {
            query
                .order_by_desc(event::Column::Timestamp)
                .order_by_desc(event::Column::Id)
        };
        let models = ordered
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to list events", Some(Box::new(e))))?;

        let mut matched = Vec::new();
        for model in models {
            let envelope = model.into_domain()?;
            if pattern.matches(&envelope.topic) {
                matched.push(envelope);
            }
        }
        let total = matched.len() as u64;
        let filtered_after_cursor: Vec<EventEnvelope> = if let Some(cursor) =
            page.cursor.as_deref().and_then(|c| Cursor::decode(c).ok())
        {
            let ts = cursor.timestamp_utc().naive_utc();
            let ascending = sort.is_ascending();
            matched
                .into_iter()
                .filter(|ev| {
                    let ev_ts = ev.timestamp.naive_utc();
                    if ascending {
                        ev_ts > ts || (ev_ts == ts && cursor.id.as_deref().is_some_and(|cid| ev.id.as_str() > cid))
                    } else {
                        ev_ts < ts || (ev_ts == ts && cursor.id.as_deref().is_some_and(|cid| ev.id.as_str() < cid))
                    }
                })
                .collect()
        } else {
            let skip = page
                .page
                .filter(|p| *p > 1)
                .map(|p| ((p - 1) * page.limit) as usize)
                .unwrap_or(0);
            matched.into_iter().skip(skip).collect()
        };

        let page_items: Vec<EventEnvelope> = filtered_after_cursor
            .into_iter()
            .take(page.limit as usize)
            .collect();
        let ids: Vec<String> = page_items.iter().map(|ev| ev.id.to_string()).collect();
        let read_map = self.fetch_read_map(reader, &ids).await?;
        let stored = page_items
            .into_iter()
            .map(|envelope| {
                let read_at = read_map.get(envelope.id.as_str()).copied();
                StoredEvent { envelope, read_at }
            })
            .collect();
        Ok((stored, total))
    }
}

#[async_trait]
impl EventStoreRepo for SeaOrmEventRepo {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn insert_event(&self, ev: &EventEnvelope) -> Outcome<()> {
        let active = event::ActiveModel::from_domain(ev);
        event::Entity::insert(active)
            .exec_without_returning(&self.db)
            .await
            .map_err(|e| Errors::db("failed to insert event", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_event_by_id(
        &self,
        scope: &OwnerScope,
        id: &Urn,
        reader_id: Option<String>,
    ) -> Outcome<Option<StoredEvent>> {
        let model = event::Entity::find()
            .filter(event::Column::Id.eq(id.to_string()))
            .filter(Self::scope_condition(scope))
            .one(&self.db)
            .await
            .map_err(|e| Errors::db("failed to query event", Some(Box::new(e))))?;

        let Some(model) = model else {
            return Ok(None);
        };
        let reader = Self::resolve_reader(scope, reader_id);
        let mut items = self.attach_read_states(vec![model], &reader).await?;
        Ok(items.pop())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn list_events(
        &self,
        scope: &OwnerScope,
        filter: &EventFilter,
        reader_id: Option<String>,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<StoredEvent>, u64)> {
        let reader = Self::resolve_reader(scope, reader_id);
        let mut query = Self::apply_scalar_filters(
            event::Entity::find().filter(Self::scope_condition(scope)),
            filter,
            &reader,
        );

        if let Some(pattern) = filter.topic_pattern()? {
            if self.db.get_database_backend() == DatabaseBackend::Sqlite {
                return self
                    .list_sqlite_with_topic(query, &pattern, &reader, page, sort)
                    .await;
            }
            query = query.filter(
                Expr::col((event::Entity, event::Column::Topic))
                    .binary(BinOper::Custom("~"), pattern.to_sql_regex()),
            );
        }

        let total = query
            .clone()
            .count(&self.db)
            .await
            .map_err(|e| Errors::db("failed to count events", Some(Box::new(e))))?;
        let models = NaiveKeyset::apply(
            query,
            page,
            sort,
            event::Column::Timestamp,
            event::Column::Id,
        )
        .all(&self.db)
        .await
        .map_err(|e| Errors::db("failed to list events", Some(Box::new(e))))?;

        let stored = self.attach_read_states(models, &reader).await?;
        Ok((stored, total))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn count_unread(&self, scope: &OwnerScope, reader_id: Option<String>) -> Outcome<u64> {
        let reader = Self::resolve_reader(scope, reader_id);
        event::Entity::find()
            .filter(Self::scope_condition(scope))
            .filter(Self::read_exists_expr(&reader).not())
            .count(&self.db)
            .await
            .map_err(|e| Errors::db("failed to count unread events", Some(Box::new(e))))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn mark_read(
        &self,
        scope: &OwnerScope,
        reader_id: Option<String>,
        id: Option<Urn>,
    ) -> Outcome<u64> {
        let reader = Self::resolve_reader(scope, reader_id);
        let mut candidates = event::Entity::find()
            .select_only()
            .column(event::Column::Id)
            .filter(Self::scope_condition(scope))
            .filter(Self::read_exists_expr(&reader).not());
        if let Some(event_urn) = id {
            candidates = candidates.filter(event::Column::Id.eq(event_urn.to_string()));
        }

        let unread_ids: Vec<String> = candidates
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| Errors::db("failed to list unread events", Some(Box::new(e))))?;

        if unread_ids.is_empty() {
            return Ok(0);
        }

        let unique_ids: Vec<String> = unread_ids
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let now = Utc::now().naive_utc();
        let models: Vec<read_state::ActiveModel> = unique_ids
            .iter()
            .map(|event_id| read_state::ActiveModel {
                event_id: ActiveValue::Set(event_id.clone()),
                reader_id: ActiveValue::Set(reader.clone()),
                read_at: ActiveValue::Set(now),
            })
            .collect();
        let count = models.len() as u64;

        match read_state::Entity::insert_many(models)
            .on_conflict(
                OnConflict::columns([read_state::Column::EventId, read_state::Column::ReaderId])
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.db)
            .await
        {
            Ok(_) => Ok(count),
            Err(DbErr::RecordNotInserted) => Ok(0),
            Err(e) => Err(Errors::db("failed to mark events read", Some(Box::new(e)))),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn insert_events_batch(&self, events: &[EventEnvelope]) -> Outcome<()> {
        if events.is_empty() {
            return Ok(());
        }
        for chunk in events.chunks(500) {
            let models: Vec<event::ActiveModel> =
                chunk.iter().map(event::ActiveModel::from_domain).collect();
            match event::Entity::insert_many(models)
                .on_conflict(OnConflict::column(event::Column::Id).do_nothing().to_owned())
                .exec(&self.db)
                .await
            {
                Ok(_) | Err(DbErr::RecordNotInserted) => {}
                Err(e) => return Err(Errors::db("failed to batch insert events", Some(Box::new(e)))),
            }
        }
        Ok(())
    }
}
