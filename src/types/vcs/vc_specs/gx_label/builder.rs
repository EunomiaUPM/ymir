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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use crate::types::present::{Missing, Present};
use super::{CompliantCredential, GxLabelCredSubject};


#[derive(Debug, Serialize, Deserialize)]
pub struct GxLabelCredSubjectBuilder<R, S, T, U> {
    pub id: Option<String>,
    #[serde(rename = "gx:labelLevel")]
    pub label_level: String,
    #[serde(rename = "gx:engine_version")]
    pub engine_version: String,
    #[serde(rename = "gx:rules_version")]
    pub rules_version: String,
    #[serde(rename = "gx:compliant_credentials")]
    pub compliant_credentials: Vec<CompliantCredential>,
    #[serde(rename = "gx:validated_criteria")]
    pub validated_criteria: Vec<String>,
    #[serde(skip)]
    _marker: PhantomData<(R, S, T, U)>,
}


impl GxLabelCredSubjectBuilder<Missing, Missing, Missing, Missing> {
    pub fn new(label_level: impl Into<String>, engine_version: impl Into<String>, rules_version: impl Into<String>, criteria: impl Into<String>) -> Self {
        GxLabelCredSubjectBuilder {
            id: None,
            label_level: label_level.into(),
            engine_version: engine_version.into(),
            rules_version: rules_version.into(),
            compliant_credentials: vec![],
            validated_criteria: vec![criteria.into()],
            _marker: PhantomData,
        }
    }
}

impl<R, S, T, U> GxLabelCredSubjectBuilder<R, S, T, U> {
    pub fn legal_person(self, vc: CompliantCredential) -> GxLabelCredSubjectBuilder<R, Present, T, U> {
        let mut compliant_credentials = self.compliant_credentials;
        compliant_credentials.push(vc);

        GxLabelCredSubjectBuilder {
            id: self.id,
            label_level: self.label_level,
            engine_version: self.engine_version,
            rules_version: self.rules_version,
            compliant_credentials,
            validated_criteria: self.validated_criteria,
            _marker: PhantomData,
        }
    }
    pub fn reg_number(self, vc: CompliantCredential) -> GxLabelCredSubjectBuilder<R, S, Present, U> {
        let mut compliant_credentials = self.compliant_credentials;
        compliant_credentials.push(vc);

        GxLabelCredSubjectBuilder {
            id: self.id,
            label_level: self.label_level,
            engine_version: self.engine_version,
            rules_version: self.rules_version,
            compliant_credentials,
            validated_criteria: self.validated_criteria,
            _marker: PhantomData,
        }
    }
    pub fn terms_cons(self, vc: CompliantCredential) -> GxLabelCredSubjectBuilder<R, S, T, Present> {
        let mut compliant_credentials = self.compliant_credentials;
        compliant_credentials.push(vc);

        GxLabelCredSubjectBuilder {
            id: self.id,
            label_level: self.label_level,
            engine_version: self.engine_version,
            rules_version: self.rules_version,
            compliant_credentials,
            validated_criteria: self.validated_criteria,
            _marker: PhantomData,
        }
    }
    pub fn id(self, id: impl Into<String>) -> GxLabelCredSubjectBuilder<Present, S, T, U> {
        GxLabelCredSubjectBuilder {
            id: Some(id.into()),
            label_level: self.label_level,
            engine_version: self.engine_version,
            rules_version: self.rules_version,
            compliant_credentials: self.compliant_credentials,
            validated_criteria: self.validated_criteria,
            _marker: PhantomData,
        }
    }
}

impl GxLabelCredSubjectBuilder<Present, Present, Present, Present> {
    pub fn build(self) -> GxLabelCredSubject {
        GxLabelCredSubject {
            id: self.id.unwrap(),
            label_level: self.label_level,
            engine_version: self.engine_version,
            rules_version: self.rules_version,
            compliant_credentials: self.compliant_credentials,
            validated_criteria: self.validated_criteria,
        }
    }
}

