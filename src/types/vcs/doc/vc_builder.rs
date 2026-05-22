/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WISSTHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABISSLISSTY or FISSTNESS FOR A PARTISSCULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  ISSf not, see <https://www.gnu.org/licenses/>.
 */

use super::super::{VcIssuer, VcType, W3cDataModelVersion};
use super::{TermsOfUse, VCEvidence, VCRefreshService, VCSchema, VCStatus, VcDocument};
use crate::types::present::{Missing, Present};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct VcDocumentBuilder<ID, ISS, CS> {
    pub context: Vec<String>,
    pub id: Option<String>,
    pub r#type: Vec<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub issuer: Option<VcIssuer>,
    pub credential_subject: Option<Value>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub credential_status: Option<VCStatus>,
    pub credential_schema: Option<Vec<VCSchema>>,
    pub refresh_service: Option<VCRefreshService>,
    pub terms_of_use: Option<TermsOfUse>,
    pub evidence: Option<Vec<VCEvidence>>,
    _marker: PhantomData<(ID, ISS, CS)>,
}

impl VcDocumentBuilder<Missing, Missing, Missing> {
    pub fn new(vc_type: &VcType, model: &W3cDataModelVersion) -> Self {
        VcDocumentBuilder {
            context: vec![model.context().to_string()],
            id: None,
            r#type: vec!["VerifiableCredential".to_string(), vc_type.to_string()],
            name: None,
            description: None,
            issuer: None,
            credential_subject: None,
            valid_from: None,
            valid_until: None,
            credential_status: None,
            credential_schema: None,
            refresh_service: None,
            terms_of_use: None,
            evidence: None,
            _marker: PhantomData,
        }
    }
}

impl<ID, ISS, CS> VcDocumentBuilder<ID, ISS, CS> {
    pub fn id(self, id: impl Into<String>) -> VcDocumentBuilder<Present, ISS, CS> {
        VcDocumentBuilder {
            context: self.context,
            id: Some(id.into()),
            r#type: self.r#type,
            name: self.name,
            description: self.description,
            issuer: self.issuer,
            credential_subject: self.credential_subject,
            valid_from: self.valid_from,
            valid_until: self.valid_until,
            credential_status: self.credential_status,
            credential_schema: self.credential_schema,
            refresh_service: self.refresh_service,
            terms_of_use: self.terms_of_use,
            evidence: self.evidence,
            _marker: PhantomData,
        }
    }
    pub fn issuer(self, issuer: VcIssuer) -> VcDocumentBuilder<ID, Present, CS> {
        VcDocumentBuilder {
            context: self.context,
            id: self.id,
            r#type: self.r#type,
            name: self.name,
            description: self.description,
            issuer: Some(issuer),
            credential_subject: self.credential_subject,
            valid_from: self.valid_from,
            valid_until: self.valid_until,
            credential_status: self.credential_status,
            credential_schema: self.credential_schema,
            refresh_service: self.refresh_service,
            terms_of_use: self.terms_of_use,
            evidence: self.evidence,
            _marker: PhantomData,
        }
    }

    pub fn credential_subject(self, credential_subject: Value) -> VcDocumentBuilder<ID, ISS, Present> {
        VcDocumentBuilder {
            context: self.context,
            id: self.id,
            r#type: self.r#type,
            name: self.name,
            description: self.description,
            issuer: self.issuer,
            credential_subject: Some(credential_subject),
            valid_from: self.valid_from,
            valid_until: self.valid_until,
            credential_status: self.credential_status,
            credential_schema: self.credential_schema,
            refresh_service: self.refresh_service,
            terms_of_use: self.terms_of_use,
            evidence: self.evidence,
            _marker: PhantomData,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn valid_from(mut self, valid_from: DateTime<Utc>) -> Self {
        self.valid_from = Some(valid_from);
        self
    }

    pub fn valid_until(mut self, valid_until: DateTime<Utc>) -> Self {
        self.valid_until = Some(valid_until);
        self
    }

    pub fn credential_status(mut self, credential_status: VCStatus) -> Self {
        self.credential_status = Some(credential_status);
        self
    }

    pub fn credential_schema(mut self, credential_schema: Vec<VCSchema>) -> Self {
        self.credential_schema = Some(credential_schema);
        self
    }

    pub fn refresh_service(mut self, refresh_service: VCRefreshService) -> Self {
        self.refresh_service = Some(refresh_service);
        self
    }

    pub fn terms_of_use(mut self, terms_of_use: TermsOfUse) -> Self {
        self.terms_of_use = Some(terms_of_use);
        self
    }

    pub fn evidence(mut self, evidence: Vec<VCEvidence>) -> Self {
        self.evidence = Some(evidence);
        self
    }
}

impl VcDocumentBuilder<Present, Present, Present> {
    pub fn build(self) -> VcDocument {
        VcDocument {
            context: self.context,
            id: self.id.unwrap(),
            r#type: self.r#type,
            name: self.name,
            description: self.description,
            issuer: self.issuer.unwrap(),
            credential_subject: self.credential_subject.unwrap(),
            valid_from: self.valid_from,
            valid_until: self.valid_until,
            credential_status: self.credential_status,
            credential_schema: self.credential_schema,
            refresh_service: self.refresh_service,
            terms_of_use: self.terms_of_use,
            evidence: self.evidence,
        }
    }
}
