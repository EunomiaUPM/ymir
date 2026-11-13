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

//! Canonical v2 envelope schema, separated UI read state, webhook delivery format and outbox.

use sea_orm_migration::prelude::*;

use crate::data::migrations::m20241123_0000001_subscriptions::Subscriptions;
use crate::data::migrations::m20260601_0000003_event_bus::Events;

/// Upgrades the event store to canonical v2 metadata and adds read-state and outbox tables.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261113_0000004_event_envelope_v2"
    }
}

/// Added columns on `subscriptions` for webhook delivery format.
#[derive(DeriveIden)]
pub enum SubscriptionsV2 {
    DeliveryFormat,
}

/// Canonical v2 metadata columns added to `events`.
#[derive(DeriveIden)]
pub enum EventsV2 {
    SpecVersion,
    Sequence,
    EventType,
    Severity,
    Outcome,
    DeliveryClass,
    PayloadType,
    DataspaceId,
    ParticipantId,
    Service,
    InstanceId,
    ActorType,
    ActorId,
    ActorParticipantId,
    ActorRole,
    SubjectKind,
    SubjectId,
    SubjectParentId,
    CausationId,
    AgreementId,
    TransferId,
    NegotiationId,
    GrantId,
    DatasetId,
    PeerId,
    PayloadDigest,
    Canonicalization,
    Attestation,
}

/// Separated per-user UI read state projection (`event_read_states`).
#[derive(DeriveIden)]
pub enum EventReadStates {
    Table,
    EventId,
    ReaderId,
    ReadAt,
}

/// Durable external broker outbox (`event_outbox`).
#[derive(DeriveIden)]
pub enum EventOutbox {
    Table,
    Id,
    SinkName,
    EventId,
    Destination,
    PartitionKey,
    EnvelopeJson,
    Attempts,
    NextRetryAt,
    LastError,
    Status,
    CreatedAt,
    DeliveredAt,
}

impl Migration {
    fn added_event_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new(EventsV2::SpecVersion)
                .string_len(16)
                .not_null()
                .default("2.0")
                .take(),
            ColumnDef::new(EventsV2::Sequence).big_integer().null().take(),
            ColumnDef::new(EventsV2::EventType)
                .string_len(32)
                .not_null()
                .default("AuditLifecycle")
                .take(),
            ColumnDef::new(EventsV2::Severity)
                .string_len(16)
                .not_null()
                .default("Info")
                .take(),
            ColumnDef::new(EventsV2::Outcome).string_len(16).null().take(),
            ColumnDef::new(EventsV2::DeliveryClass)
                .string_len(32)
                .not_null()
                .default("AuditDurable")
                .take(),
            ColumnDef::new(EventsV2::PayloadType)
                .string_len(128)
                .not_null()
                .default("legacy.event.v1")
                .take(),
            ColumnDef::new(EventsV2::DataspaceId).string_len(255).null().take(),
            ColumnDef::new(EventsV2::ParticipantId)
                .string_len(255)
                .null()
                .take(),
            ColumnDef::new(EventsV2::Service)
                .string_len(128)
                .not_null()
                .default("unknown")
                .take(),
            ColumnDef::new(EventsV2::InstanceId)
                .string_len(255)
                .not_null()
                .default("unknown")
                .take(),
            ColumnDef::new(EventsV2::ActorType)
                .string_len(32)
                .not_null()
                .default("SystemWorker")
                .take(),
            ColumnDef::new(EventsV2::ActorId)
                .string_len(255)
                .not_null()
                .default("system")
                .take(),
            ColumnDef::new(EventsV2::ActorParticipantId)
                .string_len(255)
                .null()
                .take(),
            ColumnDef::new(EventsV2::ActorRole).string_len(255).null().take(),
            ColumnDef::new(EventsV2::SubjectKind).string_len(64).null().take(),
            ColumnDef::new(EventsV2::SubjectId).string_len(255).null().take(),
            ColumnDef::new(EventsV2::SubjectParentId)
                .string_len(255)
                .null()
                .take(),
            ColumnDef::new(EventsV2::CausationId).string_len(255).null().take(),
            ColumnDef::new(EventsV2::AgreementId).string_len(255).null().take(),
            ColumnDef::new(EventsV2::TransferId).string_len(255).null().take(),
            ColumnDef::new(EventsV2::NegotiationId)
                .string_len(255)
                .null()
                .take(),
            ColumnDef::new(EventsV2::GrantId).string_len(255).null().take(),
            ColumnDef::new(EventsV2::DatasetId).string_len(255).null().take(),
            ColumnDef::new(EventsV2::PeerId).string_len(255).null().take(),
            ColumnDef::new(EventsV2::PayloadDigest)
                .string_len(128)
                .not_null()
                .default("")
                .take(),
            ColumnDef::new(EventsV2::Canonicalization)
                .string_len(32)
                .not_null()
                .default("rfc8785-sha256")
                .take(),
            ColumnDef::new(EventsV2::Attestation).json().null().take(),
        ]
    }

    fn event_indexes() -> Vec<IndexCreateStatement> {
        vec![
            Index::create()
                .name("idx_events_participant_ts")
                .table(Events::Table)
                .col(EventsV2::ParticipantId)
                .col(Events::Timestamp)
                .to_owned(),
            Index::create()
                .name("idx_events_category_outcome")
                .table(Events::Table)
                .col(EventsV2::EventType)
                .col(EventsV2::Outcome)
                .col(Events::Timestamp)
                .to_owned(),
            Index::create()
                .name("idx_events_subject")
                .table(Events::Table)
                .col(EventsV2::SubjectKind)
                .col(EventsV2::SubjectId)
                .to_owned(),
            Index::create()
                .name("idx_events_correlation")
                .table(Events::Table)
                .col(Events::CorrelationId)
                .to_owned(),
            Index::create()
                .name("idx_events_transfer_id")
                .table(Events::Table)
                .col(EventsV2::TransferId)
                .to_owned(),
            Index::create()
                .name("idx_events_agreement_id")
                .table(Events::Table)
                .col(EventsV2::AgreementId)
                .to_owned(),
            Index::create()
                .name("idx_events_grant_id")
                .table(Events::Table)
                .col(EventsV2::GrantId)
                .to_owned(),
        ]
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // One ALTER TABLE statement per column keeps SQLite and Postgres compatible.
        manager
            .alter_table(
                Table::alter()
                    .table(Subscriptions::Table)
                    .add_column(
                        ColumnDef::new(SubscriptionsV2::DeliveryFormat)
                            .string_len(32)
                            .not_null()
                            .default("envelope"),
                    )
                    .to_owned(),
            )
            .await?;

        for col in Self::added_event_columns() {
            let mut stmt = Table::alter();
            stmt.table(Events::Table).add_column(col);
            manager.alter_table(stmt).await?;
        }

        manager
            .alter_table(
                Table::alter()
                    .table(Events::Table)
                    .drop_column(Events::ReadAt)
                    .to_owned(),
            )
            .await?;

        for idx in Self::event_indexes() {
            manager.create_index(idx).await?;
        }

        manager
            .create_table(
                Table::create()
                    .table(EventReadStates::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(EventReadStates::EventId).string().not_null())
                    .col(ColumnDef::new(EventReadStates::ReaderId).string().not_null())
                    .col(
                        ColumnDef::new(EventReadStates::ReadAt)
                            .date_time()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(EventReadStates::EventId)
                            .col(EventReadStates::ReaderId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_event_read_states_event")
                            .from(EventReadStates::Table, EventReadStates::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_event_read_states_reader")
                    .table(EventReadStates::Table)
                    .col(EventReadStates::ReaderId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(EventOutbox::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(EventOutbox::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(EventOutbox::SinkName)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(EventOutbox::EventId).string().not_null())
                    .col(
                        ColumnDef::new(EventOutbox::Destination)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EventOutbox::PartitionKey)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EventOutbox::EnvelopeJson)
                            .json()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(EventOutbox::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(EventOutbox::NextRetryAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EventOutbox::LastError).string().null())
                    .col(
                        ColumnDef::new(EventOutbox::Status)
                            .string_len(32)
                            .not_null()
                            .default("Pending"),
                    )
                    .col(
                        ColumnDef::new(EventOutbox::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(EventOutbox::DeliveredAt).date_time().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_event_outbox_due")
                    .table(EventOutbox::Table)
                    .col(EventOutbox::Status)
                    .col(EventOutbox::NextRetryAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(EventOutbox::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(EventReadStates::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Events::Table)
                    .add_column(ColumnDef::new(Events::ReadAt).date_time())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Subscriptions::Table)
                    .drop_column(SubscriptionsV2::DeliveryFormat)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
