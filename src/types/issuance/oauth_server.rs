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

use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::impl_serde_via_str;
use crate::types::keys::Alg;

// ════════════════════════════════════════════════════════════════════════════════
//   AuthServerMetadata
// ════════════════════════════════════════════════════════════════════════════════

/// OAuth 2.0 Authorization Server Metadata (RFC 8414) extended for OIDC4VCI 1.0
/// (§12.3) and OIDC Discovery.
///
/// Published at `/.well-known/oauth-authorization-server`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthServerMetadata {
    // ─── RFC 8414 base ────────────────────────────────────────────────────────────
    /// URL identifying the Authorization Server. REQUIRED.
    pub issuer: String,

    /// URL of the Token Endpoint. REQUIRED for OIDC4VCI (token issuance).
    pub token_endpoint: String,

    /// URL of the Authorization Endpoint. REQUIRED if authorization_code grant is
    /// supported, otherwise OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,

    /// URL of the JWKS document. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,

    /// URL of the Dynamic Client Registration Endpoint. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,

    /// OAuth 2.0 scopes the AS supports. RECOMMENDED.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes_supported: Option<Vec<String>>,

    /// `response_type` values supported. REQUIRED if `authorization_endpoint` is
    /// present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_types_supported: Option<Vec<ResponseType>>,

    /// `response_mode` values supported. OPTIONAL (e.g. `"query"`, `"fragment"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modes_supported: Option<Vec<String>>,

    /// Grant types the AS supports. OPTIONAL. For OIDC4VCI typically includes
    /// `authorization_code` and/or the pre-authorized code URN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types_supported: Option<Vec<OidcGrantType>>,

    /// Client auth methods supported at the Token Endpoint. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_methods_supported: Option<Vec<TokenEndpointAuthMethod>>,

    /// JWS algorithms for client auth at the Token Endpoint (when using `_jwt`
    /// methods). OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_signing_alg_values_supported: Option<Vec<Alg>>,

    /// PKCE `code_challenge_method` values supported. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_challenge_methods_supported: Option<Vec<CodeChallengeMethod>>,

    // ─── OIDC Discovery extensions ────────────────────────────────────────────────
    /// Subject identifier types the AS supports. REQUIRED for OIDC compliance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_types_supported: Option<Vec<SubjectType>>,

    /// JWS algorithms supported by the AS for signing the ID Token. REQUIRED for
    /// OIDC compliance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token_signing_alg_values_supported: Option<Vec<Alg>>,

    // ─── PAR (RFC 9126) ───────────────────────────────────────────────────────────
    /// Pushed Authorization Request Endpoint URL. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pushed_authorization_request_endpoint: Option<String>,

    /// If true, the AS requires PAR for all authorization requests. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_pushed_authorization_requests: Option<bool>,

    // ─── DPoP (RFC 9449) ──────────────────────────────────────────────────────────
    /// JWS algorithms supported in DPoP proofs. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpop_signing_alg_values_supported: Option<Vec<Alg>>,

    // ─── Other OAuth endpoints ────────────────────────────────────────────────────
    /// Token Revocation Endpoint URL (RFC 7009). OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revocation_endpoint: Option<String>,

    /// Token Introspection Endpoint URL (RFC 7662). OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint: Option<String>,

    // ─── OIDC4VCI 1.0 specific ────────────────────────────────────────────────────
    /// Whether anonymous access is supported in the Pre-Authorized Code grant.
    /// OPTIONAL.
    #[serde(
        rename = "pre-authorized_grant_anonymous_access_supported",
        skip_serializing_if = "Option::is_none"
    )]
    pub pre_authorized_grant_anonymous_access_supported: Option<bool>,
}

impl AuthServerMetadata {
    /// Build an Authorization Server Metadata document focused on the
    /// Pre-Authorized Code flow.
    ///
    /// Defaults applied:
    /// - `grant_types_supported`: `[PreAuthorizedCode]`
    /// - `token_endpoint_auth_methods_supported`: `[None]` (the wallet does not
    ///   authenticate itself when redeeming a pre-authorized code).
    ///
    /// Every other field is `None`. Override any field via direct field access
    /// after construction if you need to enable other flows or features.
    pub fn new(issuer: &str, api_path: &str) -> Self {
        Self {
            issuer: issuer.to_string(),
            token_endpoint: format!("{}{}/token", issuer, api_path),

            grant_types_supported: Some(vec![OidcGrantType::PreAuthorizedCode]),
            token_endpoint_auth_methods_supported: Some(vec![TokenEndpointAuthMethod::None]),

            authorization_endpoint: None,
            // jwks_uri: Some(format!("{}{}/jwks", issuer, api_path)),
            jwks_uri: None,
            registration_endpoint: None,
            scopes_supported: None,
            response_types_supported: None,
            response_modes_supported: None,
            token_endpoint_auth_signing_alg_values_supported: None,
            code_challenge_methods_supported: None,
            subject_types_supported: None,
            id_token_signing_alg_values_supported: None,
            pushed_authorization_request_endpoint: None,
            require_pushed_authorization_requests: None,
            dpop_signing_alg_values_supported: None,
            revocation_endpoint: None,
            introspection_endpoint: None,
            pre_authorized_grant_anonymous_access_supported: None,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
//   OidcGrantType
// ════════════════════════════════════════════════════════════════════════════════

/// OAuth 2.0 grant types relevant to OIDC4VCI.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OidcGrantType {
    AuthorizationCode,
    PreAuthorizedCode,
    RefreshToken,
    ClientCredentials,
    Other(String),
}

impl Display for OidcGrantType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::AuthorizationCode => "authorization_code",
            Self::PreAuthorizedCode => "urn:ietf:params:oauth:grant-type:pre-authorized_code",
            Self::RefreshToken => "refresh_token",
            Self::ClientCredentials => "client_credentials",
            Self::Other(other) => other.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for OidcGrantType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "authorization_code" => Self::AuthorizationCode,
            "urn:ietf:params:oauth:grant-type:pre-authorized_code" => Self::PreAuthorizedCode,
            "refresh_token" => Self::RefreshToken,
            "client_credentials" => Self::ClientCredentials,
            other => Self::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(OidcGrantType);

// ════════════════════════════════════════════════════════════════════════════════
//   ResponseType
// ════════════════════════════════════════════════════════════════════════════════

/// OAuth 2.0 / OIDC response types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResponseType {
    Code,
    Token,
    IdToken,
    VpToken,
    Other(String),
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Code => "code",
            Self::Token => "token",
            Self::IdToken => "id_token",
            Self::VpToken => "vp_token",
            Self::Other(other) => other.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for ResponseType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "code" => Self::Code,
            "token" => Self::Token,
            "id_token" => Self::IdToken,
            "vp_token" => Self::VpToken,
            other => Self::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(ResponseType);

// ════════════════════════════════════════════════════════════════════════════════
//   CodeChallengeMethod
// ════════════════════════════════════════════════════════════════════════════════

/// PKCE code_challenge_method (RFC 7636).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CodeChallengeMethod {
    Plain,
    S256,
    Other(String),
}

impl Display for CodeChallengeMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Plain => "plain",
            Self::S256 => "S256",
            Self::Other(other) => other.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for CodeChallengeMethod {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "plain" => Self::Plain,
            "S256" => Self::S256,
            other => Self::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(CodeChallengeMethod);

// ════════════════════════════════════════════════════════════════════════════════
//   TokenEndpointAuthMethod
// ════════════════════════════════════════════════════════════════════════════════

/// Client authentication methods at the Token Endpoint (RFC 6749 §2.3,
/// OIDC Discovery).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenEndpointAuthMethod {
    ClientSecretBasic,
    ClientSecretPost,
    ClientSecretJwt,
    PrivateKeyJwt,
    None,
    Other(String),
}

impl Display for TokenEndpointAuthMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ClientSecretBasic => "client_secret_basic",
            Self::ClientSecretPost => "client_secret_post",
            Self::ClientSecretJwt => "client_secret_jwt",
            Self::PrivateKeyJwt => "private_key_jwt",
            Self::None => "none",
            Self::Other(other) => other.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for TokenEndpointAuthMethod {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "client_secret_basic" => Self::ClientSecretBasic,
            "client_secret_post" => Self::ClientSecretPost,
            "client_secret_jwt" => Self::ClientSecretJwt,
            "private_key_jwt" => Self::PrivateKeyJwt,
            "none" => Self::None,
            other => Self::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(TokenEndpointAuthMethod);

// ════════════════════════════════════════════════════════════════════════════════
//   SubjectType
// ════════════════════════════════════════════════════════════════════════════════

/// Subject identifier types per OIDC Core §8.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubjectType {
    Public,
    Pairwise,
    Other(String),
}

impl Display for SubjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Public => "public",
            Self::Pairwise => "pairwise",
            Self::Other(other) => other.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for SubjectType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "public" => Self::Public,
            "pairwise" => Self::Pairwise,
            other => Self::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(SubjectType);
