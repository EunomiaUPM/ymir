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

use crate::config::types::VerifyReqConfig;
use crate::types::vcs::VcType;

/// Shared behavior for evaluation contexts demanding data space verification checks.
pub trait VerifyReqConfigTrait {
    // ===== EXTRACTION ANCHORS ====================================================================

    /// Returns a backing reference to the root verification requirement configuration model.
    fn verify_req_config(&self) -> &VerifyReqConfig;

    // ===== POLICY CONTROL QUERIES ================================================================

    /// Checks whether raw cryptographic X.509 certificates are explicitly permitted within verification scopes.
    fn is_cert_allowed(&self) -> bool {
        self.verify_req_config().is_cert_allowed
    }

    /// Checks if validated incoming cryptographic certificates bypass manual authorization queues.
    fn auto_approve_cert(&self) -> bool {
        self.verify_req_config().auto_approve_cert
    }

    /// Recovers the precise array of requested Verifiable Credential taxonomy schemas.
    fn get_requested_vcs(&self) -> &[VcType] {
        &self.verify_req_config().vcs_requested
    }
}
