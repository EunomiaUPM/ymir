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

//! `events` table storing immutable canonical v2 envelopes.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::oauth::Owner;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::entities::envelope::{
    ActorType, DeliveryClass, EntityRef, EventActor, EventAttestation, EventCategory,
    EventCorrelation, EventEnvelope, EventIntegrity, EventOutcome, EventSeverity, EventSource,
    FlowRefs,
};
use crate::entities::topic::Topic;

/// `events` row.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub spec_version: String,
    pub sequence: Option<i64>,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub topic: String,
    pub event_type: String,
    pub severity: String,
    pub outcome: Option<String>,
    pub delivery_class: String,
    pub schema_version: i32,
    pub payload_type: String,
    pub dataspace_id: Option<String>,
    pub participant_id: Option<String>,
    pub service: String,
    pub instance_id: String,
    pub source_crate: String,
    pub actor_type: String,
    pub actor_id: String,
    pub actor_participant_id: Option<String>,
    pub actor_role: Option<String>,
    pub subject_kind: Option<String>,
    pub subject_id: Option<String>,
    pub subject_parent_id: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub trace_context: Option<String>,
    pub agreement_id: Option<String>,
    pub transfer_id: Option<String>,
    pub negotiation_id: Option<String>,
    pub grant_id: Option<String>,
    pub dataset_id: Option<String>,
    pub peer_id: Option<String>,
    pub payload_digest: String,
    pub canonicalization: String,
    pub attestation: Option<serde_json::Value>,
    pub payload: serde_json::Value,
    pub timestamp: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::delivery::Entity")]
    Deliveries,
    #[sea_orm(has_many = "super::read_state::Entity")]
    ReadStates,
}

impl Related<super::delivery::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deliveries.def()
    }
}

impl Related<super::read_state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReadStates.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Decodes a stored event row into its canonical v2 domain envelope.
    pub fn into_domain(self) -> Outcome<EventEnvelope> {
        let topic = Topic::new(self.topic)
            .map_err(|e| Errors::db(format!("invalid event topic: {e}"), None))?;
        let id = Urn::from_str(&self.id)
            .map_err(|e| Errors::db(format!("invalid event URN: {e}"), None))?;
        let event_type = EventCategory::from_str(&self.event_type)
            .unwrap_or(EventCategory::AuditLifecycle);
        let severity = EventSeverity::from_str(&self.severity).unwrap_or(EventSeverity::Info);
        let outcome = self
            .outcome
            .as_deref()
            .and_then(|s| EventOutcome::from_str(s).ok());
        let delivery_class =
            DeliveryClass::from_str(&self.delivery_class).unwrap_or(DeliveryClass::AuditDurable);
        let actor_type = ActorType::from_str(&self.actor_type).unwrap_or(ActorType::SystemWorker);

        let subject = match (self.subject_kind, self.subject_id) {
            (Some(kind), Some(sub_id)) => Some(EntityRef {
                kind,
                id: sub_id,
                parent_id: self.subject_parent_id,
            }),
            _ => None,
        };

        let integrity = if self.payload_digest.is_empty() {
            EventIntegrity::for_payload(&self.payload)
        } else {
            let attestation = self
                .attestation
                .and_then(|val| serde_json::from_value::<EventAttestation>(val).ok());
            EventIntegrity {
                payload_digest: self.payload_digest,
                canonicalization: self.canonicalization,
                attestation,
            }
        };

        Ok(EventEnvelope {
            id,
            spec_version: self.spec_version,
            timestamp: DateTime::from_naive_utc_and_offset(self.timestamp, Utc),
            sequence: self.sequence.map(|s| s as u64),
            topic,
            event_type,
            severity,
            outcome,
            delivery_class,
            schema_version: self.schema_version as u32,
            payload_type: self.payload_type,
            source: EventSource {
                dataspace_id: self.dataspace_id,
                participant_id: self.participant_id,
                service: self.service,
                instance_id: self.instance_id,
                crate_name: self.source_crate,
            },
            actor: EventActor {
                actor_type,
                id: self.actor_id,
                participant_id: self.actor_participant_id,
                role: self.actor_role,
            },
            subject,
            correlation: EventCorrelation {
                correlation_id: self.correlation_id,
                causation_id: self.causation_id,
                trace_context: self.trace_context,
                flow_refs: FlowRefs {
                    agreement_id: self.agreement_id,
                    transfer_id: self.transfer_id,
                    negotiation_id: self.negotiation_id,
                    grant_id: self.grant_id,
                    dataset_id: self.dataset_id,
                    peer_id: self.peer_id,
                },
            },
            integrity,
            visibility_owner: Owner::new(self.user_id, self.user_role, self.visibility),
            payload: self.payload,
        })
    }
}

impl ActiveModel {
    /// Converts a canonical v2 envelope into an insertable SeaORM model.
    pub fn from_domain(entity: &EventEnvelope) -> Self {
        let attestation = entity
            .integrity
            .attestation
            .as_ref()
            .and_then(|att| serde_json::to_value(att).ok());

        Self {
            id: ActiveValue::Set(entity.id.to_string()),
            spec_version: ActiveValue::Set(entity.spec_version.clone()),
            sequence: ActiveValue::Set(entity.sequence.map(|s| s as i64)),
            user_id: ActiveValue::Set(entity.visibility_owner.user_id.clone()),
            user_role: ActiveValue::Set(entity.visibility_owner.role.clone()),
            visibility: ActiveValue::Set(entity.visibility_owner.visibility.clone()),
            topic: ActiveValue::Set(entity.topic.to_string()),
            event_type: ActiveValue::Set(entity.event_type.as_str().to_string()),
            severity: ActiveValue::Set(entity.severity.as_str().to_string()),
            outcome: ActiveValue::Set(entity.outcome.map(|o| o.as_str().to_string())),
            delivery_class: ActiveValue::Set(entity.delivery_class.as_str().to_string()),
            schema_version: ActiveValue::Set(entity.schema_version as i32),
            payload_type: ActiveValue::Set(entity.payload_type.clone()),
            dataspace_id: ActiveValue::Set(entity.source.dataspace_id.clone()),
            participant_id: ActiveValue::Set(entity.source.participant_id.clone()),
            service: ActiveValue::Set(entity.source.service.clone()),
            instance_id: ActiveValue::Set(entity.source.instance_id.clone()),
            source_crate: ActiveValue::Set(entity.source.crate_name.clone()),
            actor_type: ActiveValue::Set(entity.actor.actor_type.as_str().to_string()),
            actor_id: ActiveValue::Set(entity.actor.id.clone()),
            actor_participant_id: ActiveValue::Set(entity.actor.participant_id.clone()),
            actor_role: ActiveValue::Set(entity.actor.role.clone()),
            subject_kind: ActiveValue::Set(entity.subject.as_ref().map(|s| s.kind.clone())),
            subject_id: ActiveValue::Set(entity.subject.as_ref().map(|s| s.id.clone())),
            subject_parent_id: ActiveValue::Set(
                entity.subject.as_ref().and_then(|s| s.parent_id.clone()),
            ),
            correlation_id: ActiveValue::Set(entity.correlation.correlation_id.clone()),
            causation_id: ActiveValue::Set(entity.correlation.causation_id.clone()),
            trace_context: ActiveValue::Set(entity.correlation.trace_context.clone()),
            agreement_id: ActiveValue::Set(entity.correlation.flow_refs.agreement_id.clone()),
            transfer_id: ActiveValue::Set(entity.correlation.flow_refs.transfer_id.clone()),
            negotiation_id: ActiveValue::Set(entity.correlation.flow_refs.negotiation_id.clone()),
            grant_id: ActiveValue::Set(entity.correlation.flow_refs.grant_id.clone()),
            dataset_id: ActiveValue::Set(entity.correlation.flow_refs.dataset_id.clone()),
            peer_id: ActiveValue::Set(entity.correlation.flow_refs.peer_id.clone()),
            payload_digest: ActiveValue::Set(entity.integrity.payload_digest.clone()),
            canonicalization: ActiveValue::Set(entity.integrity.canonicalization.clone()),
            attestation: ActiveValue::Set(attestation),
            payload: ActiveValue::Set(entity.payload.clone()),
            timestamp: ActiveValue::Set(entity.timestamp.naive_utc()),
            created_at: ActiveValue::Set(Utc::now().naive_utc()),
        }
    }
}
