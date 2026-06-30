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

use crate::config::types::ConnectionConfig;

/// Shared behavior for structural configurations managing deployment flags and operational flags.
pub trait ConnectionConfigTrait {
    // ===== EXTRACTION ANCHORS ====================================================================

    /// Returns a backing reference to the root connection configuration model.
    fn connection(&self) -> &ConnectionConfig;

    // ===== ENVIRONMENT FLAG RESOLUTION ===========================================================

    /// Evaluates whether the engine runtime is set to a production environment.
    fn is_prod(&self) -> bool {
        self.connection().is_prod
    }

    /// Evaluates whether the engine is communicating with a real production Vault instance
    /// or a mocked cryptographic provider state.
    fn is_vault_real(&self) -> bool {
        self.connection().is_vault_real
    }

    /// Evaluates if network egress/ingress points sit behind a specialized upstream TLS terminate proxy.
    fn has_tls_proxy(&self) -> bool {
        self.connection().has_tls_proxy
    }
}
