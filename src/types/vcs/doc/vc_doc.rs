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

use super::{BaseIssuer, TermsOfUse, VCEvidence, VCRefreshService, VCSchema, VCStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VcDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub r#type: Vec<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub issuer: BaseIssuer,
    #[serde(rename = "credentialSubject")]
    pub credential_subject: Value, // This is specific for each type of VC
    #[serde(
        rename = "validFrom",
        skip_serializing_if = "Option::is_none",
    )]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(
        rename = "validUntil",
        skip_serializing_if = "Option::is_none",
    )]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(rename = "credentialStatus", skip_serializing_if = "Option::is_none")]
    pub credential_status: Option<VCStatus>,
    #[serde(rename = "credentialSchema", skip_serializing_if = "Option::is_none")]
    pub credential_schema: Option<Vec<VCSchema>>,
    #[serde(rename = "refreshService", skip_serializing_if = "Option::is_none")]
    pub refresh_service: Option<VCRefreshService>,
    #[serde(rename = "termsOfUse", skip_serializing_if = "Option::is_none")]
    pub terms_of_use: Option<TermsOfUse>,
    pub evidence: Option<Vec<VCEvidence>>,
}
