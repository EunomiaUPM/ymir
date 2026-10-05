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

//! Paths of the routers ymir provides and of the endpoints its services build URLs to.
//!
//! Each module holds the routes relative to where they are mounted; a `PREFIX` only where ymir
//! itself decides the mount point. Paths with a parameter are axum templates (`/pd/{state}`):
//! routers use them as they are, and whoever builds a URL fills them with [`fill`], or takes the
//! part before the parameter with [`base`].

/// OID4VP verifier: the apps mount its router under [`verifier::PREFIX`], and the verifier
/// service builds its URIs with these paths.
pub mod verifier {
    pub const PREFIX: &str = "/verifier";
    /// Presentation definition of a verification, by its state.
    pub const PD: &str = "/pd/{state}";
    /// Where the holder's wallet posts its presentation, by state; its [`base`](super::base) is
    /// the verifier's client id.
    pub const VERIFY: &str = "/verify/{state}";
}

/// Wallet router ([`WalletRouter`](super::WalletRouter)).
pub mod wallet {
    pub const IS_LINKED: &str = "/is-linked";
    pub const LINK: &str = "/link";
    pub const KEY: &str = "/key";
    pub const KEYS: &str = "/keys";
    pub const KEY_BY_ID: &str = "/key/{id}";
    pub const DID: &str = "/did";
    pub const DID_BY_ID: &str = "/did/{id}";
    pub const DID_DEFAULT: &str = "/did/{id}/default";
    pub const DID_KEY: &str = "/did/{id}/key/{key_id}";
    pub const DID_DEFAULT_KEY: &str = "/did/{id}/key/default/{key_id}";
    pub const CREDENTIAL_BY_ID: &str = "/credential/{id}";
    pub const INFO: &str = "/info";
    pub const VCS: &str = "/vcs";
    pub const OID4VCI: &str = "/oid4vci";
    pub const OID4VP: &str = "/oid4vp";
    /// The `did:web` document, at the root of the host.
    pub const DID_DOC: &str = "/.well-known/did.json";
}

/// Health router ([`HealthRouter`](super::HealthRouter)).
pub mod health {
    pub const HEALTH: &str = "/health";
    pub const HEALTHZ: &str = "/healthz";
    pub const LIVENESS: &str = "/liveness";
    pub const READINESS: &str = "/readiness";
    pub const CIRCUITS: &str = "/health/circuits";
}

/// OpenAPI router ([`OpenapiRouter`](super::OpenapiRouter)).
pub mod openapi {
    pub const JSON: &str = "/openapi.json";
    pub const SWAGGER: &str = "/openapi";
}

/// `template` with its parameter (the first `{…}` segment) replaced by `value`; unchanged if it
/// has none.
pub fn fill(template: &str, value: &str) -> String {
    match (template.find('{'), template.find('}')) {
        (Some(open), Some(close)) if open < close => {
            format!("{}{}{}", &template[..open], value, &template[close + 1..])
        }
        _ => template.to_string(),
    }
}

/// The part of `template` before its parameter, without the trailing `/`
/// (`/verify/{state}` → `/verify`); the whole template if it has none.
pub fn base(template: &str) -> &str {
    match template.find('{') {
        Some(open) => template[..open].trim_end_matches('/'),
        None => template,
    }
}
