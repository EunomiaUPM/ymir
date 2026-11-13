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

//! Widens bounded event string columns in Postgres so DIDs (e.g. `did:jwk`) and URIs never truncate.

use sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

impl Migration {
    const TABLE_COLUMNS: &'static [(&'static str, &'static [&'static str])] = &[
        (
            "events",
            &[
                "user_role",
                "payload_type",
                "dataspace_id",
                "participant_id",
                "service",
                "instance_id",
                "actor_id",
                "actor_participant_id",
                "actor_role",
                "subject_kind",
                "subject_id",
                "subject_parent_id",
                "causation_id",
                "agreement_id",
                "transfer_id",
                "negotiation_id",
                "grant_id",
                "dataset_id",
                "peer_id",
            ],
        ),
        ("event_outbox", &["destination", "partition_key"]),
        ("subscriptions", &["user_role"]),
        ("event_deliveries", &["user_role"]),
        ("dead_letter_queue", &["user_role"]),
    ];

    fn postgres_widen_statements() -> Vec<String> {
        Self::TABLE_COLUMNS
            .iter()
            .map(|(table, cols)| {
                let alters = cols
                    .iter()
                    .map(|col| format!(r#"ALTER COLUMN "{col}" TYPE character varying"#))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(r#"ALTER TABLE "{table}" {alters}"#)
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite ignores VARCHAR(N) bounds; only Postgres enforces character varying(N) limits.
        if manager.get_database_backend() != DatabaseBackend::Postgres {
            return Ok(());
        }
        let db = manager.get_connection();
        for sql in Self::postgres_widen_statements() {
            db.execute_unprepared(&sql).await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
