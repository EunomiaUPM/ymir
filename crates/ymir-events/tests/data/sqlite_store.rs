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

//! End-to-end SeaORM tests against SQLite in-memory:
//! migrations, v2 event filters, per-reader `event_read_states`, webhook formats, and outbox.

use chrono::{Duration, Utc};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use ymir::oauth::{Owner, OwnerScope};
use ymir::paginated_spec::{Page, Sort};
use ymir::types::participants::Visibility;
use ymir_events::data::migrations::get_events_migrations;
use ymir_events::data::sea_orm::SeaOrmDataFactory;
use ymir_events::data::DataFactory;
use ymir_events::entities::commands::CreateSubscriptionDto;
use ymir_events::entities::outbox::{OutboxRecord, OutboxStatus};
use ymir_events::entities::queries::EventFilter;
use ymir_events::entities::subscription::WebhookDeliveryFormat;
use ymir_events::schemas::dataspace::DataAccessPayload;
use ymir_events::schemas::identity::{GrantAction, GrantDirection, IdentityGrantPayload};
use ymir_events::{EventContext, EventPayload, EventSource};

struct TestMigrator;

impl MigratorTrait for TestMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        get_events_migrations()
    }
}

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    TestMigrator::up(&db, None).await.unwrap();
    db
}

fn shared_owner() -> Owner {
    Owner::new(
        "user-alice",
        "/admin/upm".parse().unwrap(),
        Visibility::Public,
    )
}

#[tokio::test]
async fn per_reader_read_state_is_isolated_between_users_on_immutable_events() {
    let db = setup_db().await;
    let factory = SeaOrmDataFactory::new(db);
    let event_repo = factory.event_repository();

    let ctx = EventContext::for_owner(shared_owner())
        .with_transfer_id("tr-101")
        .with_agreement_id("ag-101");
    let source = EventSource {
        dataspace_id: Some("ds-test".to_string()),
        participant_id: Some("did:web:alice".to_string()),
        service: "dataplane".to_string(),
        instance_id: "dp-1".to_string(),
        crate_name: "dataplane".to_string(),
    };

    let access = DataAccessPayload {
        allowed: true,
        transfer_id: "tr-101".to_string(),
        agreement_id: Some("ag-101".to_string()),
        dataset_id: Some("asset-1".to_string()),
        channel: "http_proxy".to_string(),
        method: Some("GET".to_string()),
        path: Some("/".to_string()),
        status_code: Some(200),
        bytes_transferred: Some(64),
        duration_ms: None,
        peer_id: None,
        denial_reason: None,
    }
    .into_envelope(&ctx, source.clone())
    .unwrap();
    let grant_ctx = EventContext::for_owner(shared_owner());
    let denied = IdentityGrantPayload {
        action: GrantAction::Rejected,
        grant_id: "grant-1".to_string(),
        direction: GrantDirection::Received,
        kind: "consumer".to_string(),
        status: "Rejected".to_string(),
        previous_status: None,
        peer_participant_id: None,
        peer_participant_nick: None,
        vc_types: vec![],
        token_fingerprint: None,
        expires_at: None,
        reason: Some("revoked".to_string()),
    }
    .into_envelope(&grant_ctx, source)
    .unwrap();

    event_repo.insert_event(&access).await.unwrap();
    event_repo.insert_event(&denied).await.unwrap();

    let all = OwnerScope::All;
    let alice = Some("user-alice".to_string());
    let bob = Some("user-bob".to_string());

    assert_eq!(event_repo.count_unread(&all, alice.clone()).await.unwrap(), 2);
    assert_eq!(event_repo.count_unread(&all, bob.clone()).await.unwrap(), 2);

    // Alice marks one event read; Bob's unread count must remain untouched.
    let updated = event_repo
        .mark_read(&all, alice.clone(), Some(access.id.clone()))
        .await
        .unwrap();
    assert_eq!(updated, 1);

    // Repeating mark_read for Alice is idempotent (inserts 0 additional read rows).
    let repeated = event_repo
        .mark_read(&all, alice.clone(), Some(access.id.clone()))
        .await
        .unwrap();
    assert_eq!(repeated, 0);

    assert_eq!(event_repo.count_unread(&all, alice.clone()).await.unwrap(), 1);
    assert_eq!(event_repo.count_unread(&all, bob.clone()).await.unwrap(), 2);

    let fetched_alice = event_repo
        .get_event_by_id(&all, &access.id, alice.clone())
        .await
        .unwrap()
        .unwrap();
    assert!(fetched_alice.read_at.is_some());

    let fetched_bob = event_repo
        .get_event_by_id(&all, &access.id, bob.clone())
        .await
        .unwrap()
        .unwrap();
    assert!(fetched_bob.read_at.is_none());

    // Filtering by read status and canonical metadata returns accurate slices.
    let unread_for_alice = EventFilter {
        read: Some(false),
        ..Default::default()
    };
    let (items, total) = event_repo
        .list_events(&all, &unread_for_alice, alice.clone(), &Page::default(), &Sort::CreatedAtDesc)
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].envelope.id, denied.id);

    let by_transfer = EventFilter {
        transfer_id: Some("tr-101".to_string()),
        participant_id: Some("did:web:alice".to_string()),
        ..Default::default()
    };
    let (items, total) = event_repo
        .list_events(&all, &by_transfer, bob.clone(), &Page::default(), &Sort::CreatedAtDesc)
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(items[0].envelope.id, access.id);
    assert_eq!(items[0].envelope.source.dataspace_id.as_deref(), Some("ds-test"));
}

#[tokio::test]
async fn subscription_persists_delivery_format_and_topic_pattern() {
    let db = setup_db().await;
    let factory = SeaOrmDataFactory::new(db);
    let sub_repo = factory.subscription_repository();

    let created = sub_repo
        .create_subscription(
            shared_owner(),
            CreateSubscriptionDto {
                visibility: Some(Visibility::Public),
                callback_address: "https://hook.example/events".to_string(),
                topic_pattern: "transfer.*".to_string(),
                secret: None,
                headers: None,
                retry_limit: Some(4),
                delivery_format: Some(WebhookDeliveryFormat::PayloadOnly),
                expiration_time: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(created.delivery_format, WebhookDeliveryFormat::PayloadOnly);

    let matched = sub_repo
        .get_matching_subscriptions(&shared_owner(), &"transfer.started".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].delivery_format, WebhookDeliveryFormat::PayloadOnly);
}

#[tokio::test]
async fn outbox_repository_handles_enqueue_reschedule_delivery_and_dlq() {
    let db = setup_db().await;
    let factory = SeaOrmDataFactory::new(db);
    let outbox = factory.outbox_repository();

    let now = Utc::now();
    let rec1 = OutboxRecord {
        id: "ob-1".to_string(),
        sink_name: "kafka".to_string(),
        event_id: "urn:uuid:11111111-1111-1111-1111-111111111111".to_string(),
        destination: "dataspace.events".to_string(),
        partition_key: "ag-1".to_string(),
        envelope_json: serde_json::json!({ "ok": true }),
        attempts: 1,
        next_retry_at: now - Duration::seconds(5),
        last_error: Some("timeout".to_string()),
        status: OutboxStatus::Pending,
        created_at: now,
        delivered_at: None,
    };
    let rec2 = OutboxRecord {
        id: "ob-2".to_string(),
        next_retry_at: now + Duration::seconds(60),
        ..rec1.clone()
    };

    outbox.enqueue(&rec1).await.unwrap();
    outbox.enqueue(&rec2).await.unwrap();

    let due = outbox.get_due_pending(now, 10).await.unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].id, "ob-1");

    outbox.mark_delivered("ob-1", 2).await.unwrap();
    outbox
        .reschedule("ob-2", 2, now - Duration::seconds(1), "broker down")
        .await
        .unwrap();

    let due_after = outbox.get_due_pending(now, 10).await.unwrap();
    assert_eq!(due_after.len(), 1);
    assert_eq!(due_after[0].id, "ob-2");
    assert_eq!(due_after[0].attempts, 2);

    outbox.mark_dead_letter("ob-2", 3, "permanent").await.unwrap();
    assert!(outbox.get_due_pending(now, 10).await.unwrap().is_empty());
}
