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

use crate::data::entities::received::verification::{Model, Plan};
use crate::errors::Outcome;
use crate::types::vcs::VPDef;
use async_trait::async_trait;

/// Verifiable Presentation verification service.
///
/// Responsible for generating presentation definitions,
/// creating verification requests and validating received
/// VP tokens against the requested requirements.
#[async_trait]
pub trait VerifierTrait: Send + Sync + 'static {
    /// Creates a new verification plan associated with a grant.
    ///
    /// The resulting [`Plan`] establishes the expected cryptographic audience
    /// (the Verifier's endpoint) and the array of allowed VC types.
    fn build_vp_plan(&self, id: &str) -> Outcome<Plan>;

    /// Generates the wallet-facing verification URI used to
    /// initiate the presentation flow.
    ///
    /// Compiles an `openid4vp://` scheme deployment utilizing `direct_post` mode
    /// and points the wallet to the ephemeral presentation definition endpoint.
    fn generate_verification_uri(&self, verification_model: &Model) -> String;

    /// Builds the Presentation Definition describing the
    /// credentials that must be presented.
    ///
    /// Follows the DIF Presentation Exchange specification to restrict
    /// the submission to the requested types within the [`Model`].
    fn generate_vpd(&self, verification_model: &Model) -> Outcome<VPDef>;

    /// Verifies all received presentations and updates the
    /// verification model with the validation results.
    ///
    /// This validates the outer VP envelope (nonce, holder signature, expiration)
    /// as well as each nested Verifiable Credential inside the token. Updates
    /// the mutable [`Model`] status to reflect success or failure.
    async fn verify_all(&self, verification_model: &mut Model, vp_token: &str) -> Outcome<()>;
}
