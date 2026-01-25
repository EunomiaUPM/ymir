/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use super::WaltIdConfigTrait;
use crate::config::traits::SingleHostTrait;
use crate::config::types::HostConfig;
use crate::errors::{ErrorLogTrait, Errors};
use crate::types::dids::did_config::DidConfig;
use crate::types::dids::did_type::DidType;
use crate::types::wallet::WalletConfig;
use tracing::error;

pub struct WaltIdConfig {
    pub host: HostConfig,
    pub ssi_wallet_config: WalletConfig,
    pub did_config: DidConfig,
}

impl WaltIdConfigTrait for WaltIdConfig {
    fn get_raw_wallet_config(&self) -> WalletConfig {
        self.ssi_wallet_config.clone()
    }
    fn get_wallet_host(&self) -> String {
        self.ssi_wallet_config.api.get_host()
    }
    fn get_host(&self) -> String {
        self.host.get_host()
    }
    fn get_did_type(&self) -> DidType {
        self.did_config.r#type.clone()
    }
    fn get_did_web_path(&self) -> Option<String> {
        match self.did_config.r#type {
            DidType::Web => self.did_config.did_web_options.as_ref()?.path.clone(),
            _ => {
                let error = Errors::module_new("didweb");
                error!("{}", error.log());
                None
            }
        }
    }
    fn get_did_web_domain(&self) -> String {
        let domain = match self.did_config.r#type {
            DidType::Web => {
                Some(self.did_config.did_web_options.as_ref().expect("didweb").domain.clone())
            }
            _ => {
                let error = Errors::module_new("didweb");
                error!("{}", error.log());
                None
            }
        };

        domain.expect("didweb")
    }
}
