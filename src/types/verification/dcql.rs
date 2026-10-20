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

//! Digital Credentials Query Language (DCQL), how an OpenID4VP 1.0 Verifier asks for credentials
//! (OpenID4VP 1.0 §6). Only what this stack uses is typed; the rest travels as JSON.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::vcs::{VcFormat, VcType};

/// A DCQL query (§6): the credentials requested and, optionally, which combinations of them
/// satisfy the request.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DcqlQuery {
    pub credentials: Vec<CredentialQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_sets: Option<Vec<CredentialSetQuery>>,
}

/// One requested credential (§6.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CredentialQuery {
    /// Unique in the request; letters, digits, `_` and `-`. Keys the presentations in the
    /// response's `vp_token`.
    pub id: String,
    /// Credential Format Identifier (Appendix B), e.g. `jwt_vc_json` ([`VcFormat::JwtVcJson`]).
    pub format: VcFormat,
    /// Whether more than one credential may answer this query (default `false`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub multiple: bool,
    /// Format-specific constraints; for W3C VCs, [`CredentialQueryMeta::type_values`].
    pub meta: CredentialQueryMeta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claims: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_sets: Option<Vec<Vec<String>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted_authorities: Option<Vec<Value>>,
    /// Whether the Verifier wants a proof that the presenter holds the credential (default
    /// `true`): for W3C VCs, a Verifiable Presentation.
    #[serde(
        default = "CredentialQuery::default_holder_binding",
        skip_serializing_if = "CredentialQuery::is_default_holder_binding"
    )]
    pub require_cryptographic_holder_binding: bool,
}

/// The `meta` of a W3C VC credential query (B.1.1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CredentialQueryMeta {
    /// Alternative sets of types: a credential matches when its `type` holds every type of at
    /// least one set (in any order, with any other types).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_values: Option<Vec<Vec<String>>>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Which credential queries, together, satisfy the request (§6.2).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CredentialSetQuery {
    pub options: Vec<Vec<String>>,
    #[serde(default = "CredentialQuery::default_holder_binding")]
    pub required: bool,
}

impl DcqlQuery {
    /// One `jwt_vc_json` query per requested type, all of them required.
    pub fn for_types(types: &[VcType]) -> Self {
        Self {
            credentials: types.iter().map(CredentialQuery::for_type).collect(),
            credential_sets: None,
        }
    }

    /// The query with `id`, if any.
    pub fn query(&self, id: &str) -> Option<&CredentialQuery> {
        self.credentials.iter().find(|q| q.id == id)
    }
}

impl CredentialQuery {
    /// A single `jwt_vc_json` credential of type `vc_type`, presented with holder binding.
    pub fn for_type(vc_type: &VcType) -> Self {
        let type_name = vc_type.to_string();
        Self {
            id: Self::id_for(&type_name),
            format: VcFormat::JwtVcJson,
            multiple: false,
            meta: CredentialQueryMeta {
                type_values: Some(vec![vec![type_name]]),
                extra: Default::default(),
            },
            claims: None,
            claim_sets: None,
            trusted_authorities: None,
            require_cryptographic_holder_binding: true,
        }
    }

    /// A query id from a type name: anything but letters, digits, `_` and `-` becomes `_`
    /// (`gx:LabelCredential` → `gx_LabelCredential`).
    pub fn id_for(type_name: &str) -> String {
        type_name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect()
    }

    /// Whether a credential with these `types` answers the query: no `type_values` means any.
    pub fn matches_types(&self, types: &[String]) -> bool {
        match &self.meta.type_values {
            None => true,
            Some(sets) => sets
                .iter()
                .any(|set| set.iter().all(|wanted| types.iter().any(|t| t == wanted))),
        }
    }

    fn default_holder_binding() -> bool {
        true
    }

    fn is_default_holder_binding(value: &bool) -> bool {
        *value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The query for a type is a single required `jwt_vc_json` credential with that type.
    #[test]
    fn builds_one_query_per_type() {
        let query = DcqlQuery::for_types(&[VcType::DataspaceParticipant, VcType::GxLabel]);
        let json = serde_json::to_value(&query).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "credentials": [
                    { "id": "DataSpaceParticipant", "format": "jwt_vc_json",
                      "meta": { "type_values": [["DataSpaceParticipant"]] } },
                    { "id": "gx_LabelCredential", "format": "jwt_vc_json",
                      "meta": { "type_values": [["gx:LabelCredential"]] } }
                ]
            })
        );
        let back: DcqlQuery = serde_json::from_value(json).unwrap();
        assert_eq!(back, query);
        assert!(back.credentials[0].require_cryptographic_holder_binding);
    }

    /// A credential matches when it holds every type of some set, whatever else it holds.
    #[test]
    fn matches_type_sets() {
        let query = CredentialQuery::for_type(&VcType::DataspaceParticipant);
        let types = |t: &[&str]| t.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(query.matches_types(&types(&["VerifiableCredential", "DataSpaceParticipant"])));
        assert!(!query.matches_types(&types(&["VerifiableCredential"])));
    }
}
