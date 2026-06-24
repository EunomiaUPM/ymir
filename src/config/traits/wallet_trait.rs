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
    
use crate::config::traits::HostsConfigTrait;
use crate::config::types::{HostType, WalletConfig};
use crate::types::wallet::WalletInstance;

/// Shared behavior for component managers overseeing user wallet instance states.
pub trait WalletConfigTrait {
    // ===== EXTRACTION ANCHORS ====================================================================

    /// Returns a backing reference to the root wallet configuration model.
    fn wallet_config(&self) -> &WalletConfig;

    // ===== METRIC ROUTING QUERIES ================================================================

    /// Computes the complete target endpoint URI route for the wallet service API surface.
    fn get_wallet_api_url(&self, host: HostType) -> String {
        self.wallet_config().api.get_host(host)
    }

    /// Recovers a direct reference to the current structural runtime [`WalletInstance`].
    fn get_wallet(&self) -> &WalletInstance {
        &self.wallet_config().wallet
    }
}