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

mod authorization_request;
pub mod dcql;
mod direct_post_response;
mod draft20_request;
mod presentation_request;
pub mod input_descriptor;
pub mod request_object;
mod status;
mod verify_payload;
pub mod vp_def;
mod vp_doc;
mod request_passing;

pub use authorization_request::{AuthorizationRequest, RequestReference};
pub use dcql::{CredentialQuery, CredentialQueryMeta, CredentialSetQuery, DcqlQuery};
pub use direct_post_response::DirectPostResponse;
pub use draft20_request::{DefinitionSource, Draft20Request};
pub use request_object::{RequestObjectPayload, RequestUriPost};
pub use presentation_request::{PresentationRequest, Requested, UNTRUSTED_REQUEST, VerifierProof};
pub use status::VerificationStatus;
pub use verify_payload::VerifyPayload;
pub use vp_doc::VpDocument;
pub use request_passing::RequestPassing;

/// Client Identifier Prefix of an unsigned request whose client is identified by its response
/// URI (OpenID4VP 1.0 §5.9.3): the Client Identifier is `redirect_uri:` + `response_uri`.
pub const REDIRECT_URI_PREFIX: &str = "redirect_uri:";

/// Client Identifier Prefix of a Verifier identified by its DID (§5.9.3): the Client Identifier
/// is `decentralized_identifier:` + the DID, and its requests are signed with a key of the DID.
pub const DECENTRALIZED_IDENTIFIER_PREFIX: &str = "decentralized_identifier:";

/// Response mode where the wallet posts its answer to `response_uri` as a form (§8.2).
pub const DIRECT_POST: &str = "direct_post";