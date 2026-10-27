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
use crate::types::verification::{
    DECENTRALIZED_IDENTIFIER_PREFIX, DIRECT_POST, REDIRECT_URI_PREFIX,
};
use crate::types::jwt::Jwt;
use crate::types::verification::{DcqlQuery, RequestObjectPayload};
use crate::utils::Verified;

use super::{PresentationRequest, Requested, VerifierProof};
use crate::utils::get_query_param;

/// What the wallet keeps of an OpenID4VP 1.0 Authorization Request.
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    /// Full Client Identifier, prefix included: the `aud` of every presentation.
    pub client_id: String,
    /// Where the answer is posted, as the verifier sent it.
    pub response_uri: String,
    pub nonce: String,
    /// Echoed back with the answer when present.
    pub state: Option<String>,
    pub dcql_query: DcqlQuery,
}

/// The URI of a request passed by reference (§5.10): who asks, and where its signed Request
/// Object is.
#[derive(Debug, Clone)]
pub struct RequestReference {
    /// Full Client Identifier: `decentralized_identifier:<did>`, the only prefix accepted here.
    pub client_id: String,
    pub request_uri: String,
    /// `request_uri_method=post`: fetch with a POST carrying a `wallet_nonce`.
    pub post: bool,
}

impl RequestReference {
    /// The reference in `uri`, or `None` when the request comes by value.
    pub fn parse(uri: &str) -> Outcome<Option<Self>> {
        let parsed = Url::parse(uri)
            .map_err(|e| Errors::parse(format!("Invalid request URI: {uri}"), Some(Box::new(e))))?;
        let Ok(request_uri) = get_query_param(&parsed, "request_uri") else {
            return Ok(None);
        };

        let client_id = get_query_param(&parsed, "client_id")?;
        if !client_id.starts_with(DECENTRALIZED_IDENTIFIER_PREFIX) {
            return Err(Errors::not_impl(
                format!("Signed requests from '{client_id}' are not supported, only DIDs"),
                None,
            ));
        }
        let post = match get_query_param(&parsed, "request_uri_method") {
            Ok(method) if method == "post" => true,
            Ok(method) if method == "get" => false,
            Ok(method) => {
                return Err(Errors::format(
                    BadFormat::Received,
                    format!("Unknown request_uri_method '{method}'"),
                    None,
                ));
            }
            Err(_) => false,
        };

        Ok(Some(Self {
            client_id,
            request_uri,
            post,
        }))
    }

    /// The verifier's DID, from the Client Identifier.
    pub fn did(&self) -> &str {
        self.client_id
            .strip_prefix(DECENTRALIZED_IDENTIFIER_PREFIX)
            .unwrap_or(&self.client_id)
    }
}

impl From<AuthorizationRequest> for PresentationRequest {
    fn from(request: AuthorizationRequest) -> Self {
        PresentationRequest {
            client_id: request.client_id,
            response_uri: request.response_uri,
            nonce: request.nonce,
            state: request.state,
            requested: Requested::Dcql(request.dcql_query),
            verifier: VerifierProof::Unsigned,
        }
    }
}

impl AuthorizationRequest {
    /// Whether `uri` is an OpenID4VP 1.0 request (a DCQL query by value, or a `request_uri`)
    /// rather than a draft 20 one (Presentation Definition).
    pub fn is_1_0(uri: &str) -> bool {
        Url::parse(uri).is_ok_and(|url| {
            url.query_pairs()
                .any(|(k, _)| k == "dcql_query" || k == "request_uri")
        })
    }

    /// The request a verified Request Object carries, fetched through `reference`. Only what
    /// this wallet can answer is accepted: `vp_token` over `direct_post`.
    pub fn from_request_object(
        request: &Verified<Jwt<RequestObjectPayload>>,
        reference: &RequestReference,
    ) -> Outcome<Self> {
        let body = request.body();
        if body.client_id != reference.client_id {
            return Err(Errors::security(
                "The Request Object names another client_id than the request URI",
                None,
            ));
        }
        expect(&body.response_type, "vp_token", "response_type")?;
        expect(&body.response_mode, DIRECT_POST, "response_mode")?;
        let nonce = request
            .claims()
            .nonce()
            .ok_or_else(|| Errors::security("The Request Object lacks a nonce", None))?;

        Ok(Self {
            client_id: body.client_id.clone(),
            response_uri: body.response_uri.clone(),
            nonce: nonce.to_string(),
            state: body.state.clone(),
            dcql_query: body.dcql_query.clone(),
        })
    }

    /// Reads and checks a request passed by value. Only what this wallet can answer is accepted:
    /// `vp_token` over `direct_post`, from an unsigned `redirect_uri:` client whose identifier
    /// is its response URI (§5.9.3).
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

        if optional("request").is_some() {
            return Err(Errors::not_impl(
                "Signed OpenID4VP requests by value (JAR `request`) are not supported",
                None,
            ));
        }
        expect(&param("response_type")?, "vp_token", "response_type")?;
        expect(&param("response_mode")?, DIRECT_POST, "response_mode")?;

        let client_id = param("client_id")?;
        let response_uri = param("response_uri")?;
        match client_id.strip_prefix(REDIRECT_URI_PREFIX) {
            Some(target) if target == response_uri => {}
            Some(_) => {
                return Err(Errors::format(
                    BadFormat::Received,
                    "client_id must be redirect_uri: followed by the response_uri",
                    None,
                ));
            }
            None => {
                return Err(Errors::not_impl(
                    format!("Client Identifier Prefix of '{client_id}' is not supported"),
                    None,
                ));
            }
        }

        let dcql_query = serde_json::from_str(&param("dcql_query")?).map_err(|e| {
            Errors::parse("dcql_query is not a valid DCQL query", Some(Box::new(e)))
        })?;

        Ok(Self {
            client_id,
            response_uri,
            nonce: param("nonce")?,
            state: optional("state"),
            dcql_query,
        })
    }
}

fn expect(value: &str, wanted: &str, name: &str) -> Outcome<()> {
    if value != wanted {
        return Err(Errors::not_impl(
            format!("{name} '{value}' is not supported, only '{wanted}'"),
            None,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use reqwest::Url;

    use super::{AuthorizationRequest, RequestReference};

    fn uri(client_id: &str, response_uri: &str) -> String {
        let query =
            r#"{"credentials":[{"id":"q","format":"jwt_vc_json","meta":{"type_values":[["T"]]}}]}"#;
        Url::parse_with_params(
            "openid4vp://authorize",
            [
                ("response_type", "vp_token"),
                ("client_id", client_id),
                ("response_mode", "direct_post"),
                ("response_uri", response_uri),
                ("nonce", "n-1"),
                ("state", "s-1"),
                ("dcql_query", query),
            ],
        )
        .unwrap()
        .to_string()
    }

    /// A request names its response URI as client and carries the query.
    #[test]
    fn reads_a_redirect_uri_request() {
        let target = "http://verifier/api/v1/verifier/verify/s-1";
        let uri = uri(&format!("redirect_uri:{target}"), target);
        assert!(AuthorizationRequest::is_1_0(&uri));

        let request = AuthorizationRequest::parse(&uri).unwrap();
        assert_eq!(request.response_uri, target);
        assert_eq!(request.state.as_deref(), Some("s-1"));
        assert_eq!(request.dcql_query.credentials[0].id, "q");
    }

    /// A request by reference names a DID client and where to fetch its Request Object.
    #[test]
    fn reads_a_request_by_reference() {
        let uri = Url::parse_with_params(
            "openid4vp://authorize",
            [
                ("client_id", "decentralized_identifier:did:web:verifier"),
                ("request_uri", "http://verifier/api/v1/verifier/request/s-1"),
                ("request_uri_method", "post"),
            ],
        )
        .unwrap()
        .to_string();
        assert!(AuthorizationRequest::is_1_0(&uri));

        let reference = RequestReference::parse(&uri).unwrap().unwrap();
        assert_eq!(reference.did(), "did:web:verifier");
        assert!(reference.post);

        let by_value = uri.replace("request_uri", "other");
        assert!(RequestReference::parse(&by_value).unwrap().is_none());
    }

    /// The client must be the response URI itself.
    #[test]
    fn refuses_a_client_other_than_the_response_uri() {
        let uri = uri("redirect_uri:http://elsewhere", "http://verifier/verify/s-1");
        assert!(AuthorizationRequest::parse(&uri).is_err());
    }
}
