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

use reqwest::Url;

use crate::errors::{BadFormat, Errors, Outcome};
use crate::types::vcs::VPDef;
use crate::utils::get_query_param;

/// Where a draft 20 request puts its Presentation Definition.
#[derive(Debug)]
pub enum DefinitionSource {
    ByValue(VPDef),
    ByReference(String),
}

/// What an OpenID4VP draft 20 request URI carries.
#[derive(Debug)]
pub struct Draft20Request {
    pub client_id: String,
    pub client_id_scheme: Option<String>,
    pub response_type: String,
    pub response_mode: String,
    /// Where the answer is posted, as the verifier sent it.
    pub response_uri: String,
    pub nonce: String,
    /// Echoed back with the answer when present.
    pub state: Option<String>,
    pub definition: DefinitionSource,
}

impl Draft20Request {
    /// Whether `uri` carries a Presentation Definition, by value or by reference.
    pub fn is_draft20(uri: &str) -> bool {
        Url::parse(uri).is_ok_and(|url| {
            url.query_pairs().any(|(k, _)| {
                k == "presentation_definition" || k == "presentation_definition_uri"
            })
        })
    }

    pub fn parse(uri: &str) -> Outcome<Self> {
        let parsed = Url::parse(uri)
            .map_err(|e| Errors::parse(format!("Invalid request URI: {uri}"), Some(Box::new(e))))?;
        let param = |name: &str| get_query_param(&parsed, name);
        let optional = |name: &str| {
            parsed
                .query_pairs()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.into_owned())
        };

        let definition = if let Some(json) = optional("presentation_definition") {
            DefinitionSource::ByValue(serde_json::from_str(&json).map_err(|e| {
                Errors::parse("presentation_definition is not valid JSON", Some(Box::new(e)))
            })?)
        } else if let Some(pd_uri) = optional("presentation_definition_uri") {
            DefinitionSource::ByReference(pd_uri)
        } else {
            return Err(Errors::format(
                BadFormat::Received,
                "URI lacks both presentation_definition and presentation_definition_uri",
                None,
            ));
        };

        Ok(Self {
            client_id: param("client_id")?,
            client_id_scheme: optional("client_id_scheme"),
            response_type: param("response_type")?,
            response_mode: param("response_mode")?,
            response_uri: param("response_uri")?,
            nonce: param("nonce")?,
            state: optional("state"),
            definition,
        })
    }
}
