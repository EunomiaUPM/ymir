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

//! Postgres-specific integration test verifying long DIDs (`did:jwk`) and repo queries.

use chrono::{Duration, Utc};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use ymir::oauth::OwnerScope;
use ymir::paginated_spec::{Page, Sort};
use ymir::types::oauth::UserInfo;
use ymir::types::participants::Visibility;
use ymir_events::data::migrations::get_events_migrations;
use ymir_events::data::sea_orm::SeaOrmDataFactory;
use ymir_events::data::DataFactory;
use ymir_events::entities::outbox::{OutboxRecord, OutboxStatus};
use ymir_events::entities::queries::EventFilter;
use ymir_events::schemas::dataspace::{KeystoreAuditPayload, ResourceAction};
use ymir_events::{EventActor, EventContext, EventPayload, EventSource};

struct PgTestMigrator;

impl MigratorTrait for PgTestMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        get_events_migrations()
    }
}

fn long_did_keystore_envelope(
    ctx: &EventContext,
    dataspace_id: Option<String>,
    long_did: String,
    instance_id: String,
    key: &str,
) -> ymir_events::EventEnvelope {
    let source = EventSource {
        dataspace_id,
        participant_id: Some(long_did),
        service: "control-plane".to_string(),
        instance_id,
        crate_name: "keystore".to_string(),
    };
    KeystoreAuditPayload {
        action: ResourceAction::Created,
        entry_kind: "parameter".to_string(),
        entry_id: key.to_string(),
        name: key.to_string(),
        key_type: None,
        value_redacted: true,
    }
    .into_envelope(ctx, source)
    .unwrap()
}

async fn connect_isolated_schema(schema: &str) -> DatabaseConnection {
    let url = std::env::var("YMIR_TEST_DATABASE_URL")
        .expect("YMIR_TEST_DATABASE_URL must be set to run ignored postgres tests");
    let mut opts = ConnectOptions::new(url);
    opts.max_connections(1);
    let db = Database::connect(opts).await.unwrap();
    // Single-connection pool preserves search_path across migrations and repo calls.
    db.execute_unprepared(&format!("DROP SCHEMA IF EXISTS {schema} CASCADE"))
        .await
        .unwrap();
    db.execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    db.execute_unprepared(&format!("SET search_path TO {schema}"))
        .await
        .unwrap();
    db
}

#[tokio::test]
#[ignore = "requires Postgres via YMIR_TEST_DATABASE_URL"]
async fn postgres_handles_long_did_jwk_identifiers_and_queries() {
    let db = connect_isolated_schema("ymir_test_long_did").await;
    PgTestMigrator::up(&db, None).await.unwrap();

    // RSA `did:jwk` strings exceed 500 characters and must never fail varchar(255) checks.
    let long_did = format!("did:jwk:{}", "A".repeat(520));
    let user = UserInfo::system();
    let ctx = EventContext::for_user_with_visibility(&user, Visibility::Private)
        .with_peer_id(long_did.clone())
        .with_actor(EventActor::peer(long_did.clone(), Some("/admin".to_string())));

    let env = long_did_keystore_envelope(
        &ctx,
        Some(format!("urn:dataspace:{}", "x".repeat(300))),
        long_did.clone(),
        format!("pod-{}", "n".repeat(300)),
        "/test/key",
    );

    let factory = SeaOrmDataFactory::new(db.clone());
    let events = factory.event_repository();
    events.insert_event(&env).await.unwrap();

    let scope = OwnerScope::seeing(&user);
    let filter = EventFilter {
        topic: Some("keystore_parameters:**".parse().unwrap()),
        participant_id: Some(long_did.clone()),
        ..Default::default()
    };
    let (items, total) = events
        .list_events(&scope, &filter, Some("system".to_string()), &Page::default(), &Sort::CreatedAtDesc)
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].envelope.source.participant_id.as_deref(), Some(long_did.as_str()));

    assert_eq!(events.count_unread(&scope, Some("system".to_string())).await.unwrap(), 1);
    assert_eq!(
        events.mark_read(&scope, Some("system".to_string()), Some(env.id.clone())).await.unwrap(),
        1
    );
    assert_eq!(events.count_unread(&scope, Some("system".to_string())).await.unwrap(), 0);
    let fetched = events
        .get_event_by_id(&scope, &env.id, Some("system".to_string()))
        .await
        .unwrap()
        .unwrap();
    assert!(fetched.read_at.is_some());
    assert!(serde_json::to_value(&fetched.envelope).is_ok());

    let unread_filter = EventFilter {
        read: Some(false),
        ..Default::default()
    };
    let (_, unread_total) = events
        .list_events(&scope, &unread_filter, Some("system".to_string()), &Page::default(), &Sort::CreatedAtDesc)
        .await
        .unwrap();
    assert_eq!(unread_total, 0);

    let read_filter = EventFilter {
        read: Some(true),
        ..Default::default()
    };
    let (_, read_total) = events
        .list_events(&scope, &read_filter, Some("system".to_string()), &Page::default(), &Sort::CreatedAtDesc)
        .await
        .unwrap();
    assert_eq!(read_total, 1);

    // Shrinking back to varchar(255) must fail now that a >500-char DID row is stored.
    db.execute_unprepared(r#"ALTER TABLE "events" ALTER COLUMN "participant_id" TYPE varchar(255)"#)
        .await
        .unwrap_err();
    assert_eq!(events.mark_read(&scope, Some("system".to_string()), None).await.unwrap(), 0);

    let now = Utc::now();
    let outbox = factory.outbox_repository();
    outbox
        .enqueue(&OutboxRecord {
            id: "ob-pg-1".to_string(),
            sink_name: "kafka".to_string(),
            event_id: env.id.to_string(),
            destination: format!("https://example.org/{}", "path/".repeat(70)),
            partition_key: long_did,
            envelope_json: serde_json::to_value(&env).unwrap(),
            attempts: 0,
            next_retry_at: now - Duration::seconds(1),
            last_error: None,
            status: OutboxStatus::Pending,
            created_at: now,
            delivered_at: None,
        })
        .await
        .unwrap();

    db.execute_unprepared("SELECT 1").await.unwrap();
}

#[tokio::test]
#[ignore = "requires Postgres via YMIR_TEST_DATABASE_URL"]
async fn postgres_widen_migration_upgrades_existing_varchar_255_schema() {
    let db = connect_isolated_schema("ymir_test_widen").await;
    let migrations = get_events_migrations();
    PgTestMigrator::up(&db, Some((migrations.len() - 1) as u32)).await.unwrap();

    db.execute_unprepared(
        r#"ALTER TABLE "events" ALTER COLUMN "participant_id" TYPE varchar(255), ALTER COLUMN "actor_id" TYPE varchar(255)"#,
    )
    .await
    .unwrap();
    db.execute_unprepared(
        r#"ALTER TABLE "event_outbox" ALTER COLUMN "partition_key" TYPE varchar(255)"#,
    )
    .await
    .unwrap();

    PgTestMigrator::up(&db, None).await.unwrap();

    let long_did = format!("did:jwk:{}", "B".repeat(520));
    let user = UserInfo::system();
    let ctx = EventContext::for_user_with_visibility(&user, Visibility::Private);
    let env = long_did_keystore_envelope(&ctx, None, long_did, "node-1".to_string(), "/upgrade/test");

    SeaOrmDataFactory::new(db)
        .event_repository()
        .insert_event(&env)
        .await
        .unwrap();
}
