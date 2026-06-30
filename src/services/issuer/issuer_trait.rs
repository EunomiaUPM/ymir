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

use crate::data::entities::shared::issuance;
use crate::errors::Outcome;
use crate::types::gnap::grant_request::GrantRequestKind;
use crate::types::gnap::grant_request::client::Client;
use crate::types::issuance::{
    AuthServerMetadata, CredentialRequest, IssuerMetadata, IssuingToken, VcCredOffer,
    VcTransmissionOffer,
};
use crate::types::jwt::VCJwtClaims;
use crate::types::vcs::{VcType, VcTypeConfig};
use async_trait::async_trait;

/// OpenID4VCI Verifiable Credential Issuer service specification.
///
/// Defines the core contract for managing the credential issuance lifecycle, covering
/// cryptographic handshake validations, metadata compilation, and secure signature generation.
#[async_trait]
pub trait IssuerTrait: Send + Sync + 'static {
    // ===== ISSUANCE INITIALIZATION & OFFERS ======================================================

    /// Provisions an internal transactional issuance plan derived from an authenticated client request.
    async fn build_issuance_plan(
        &self,
        id: &str,
        grant_request_kind: GrantRequestKind,
        client: Client,
        available_vcs: &[VcType],
    ) -> Outcome<issuance::Plan>;

    /// Compiles token payload data necessary to build a pre-authorized credential offer.
    fn get_cred_offer_data(&self, model: &issuance::Model) -> VcCredOffer;

    /// Generates a standard-compliant `openid-credential-offer://` URI wrapper.
    ///
    /// Depending on the [`VcTransmissionOffer`] configuration, embeds the payload
    /// either inline (`ByValue`) or as a short-lived URL query reference (`ByReference`).
    fn generate_issuing_uri(&self, offer_type: VcTransmissionOffer) -> Outcome<String>;

    // ===== METADATA DISCOVERY ====================================================================

    /// Compiles the static standard `.well-known/openid-credential-issuer` metadata registry.
    fn get_issuer_metadata(&self, vcs: &[VcType]) -> IssuerMetadata;

    /// Compiles the standard metadata describing the backing OAuth 2.0 / GNAP Authorization Server.
    fn get_oauth_server_data(&self) -> AuthServerMetadata;

    // ===== SECURITY VALIDATION & SIGNING =========================================================

    /// Formulates a valid access [`IssuingToken`] package containing session lifetimes.
    fn get_token(&self, model: &issuance::Model) -> IssuingToken;

    /// Validates the client's payload request token against the session state and asserts the Proof of Possession (PoP).
    async fn validate_cred_req(
        &self,
        issuance: &issuance::Model,
        cred_req: CredentialRequest,
        token: &str,
    ) -> Outcome<(String, VcTypeConfig)>;

    /// Digitally signs the structured credential claims using asymmetric keys pulled securely from the Vault.
    async fn sign_claims(&self, claims: &VCJwtClaims) -> Outcome<String>;
}
