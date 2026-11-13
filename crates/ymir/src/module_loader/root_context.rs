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

//! Process-wide infrastructure built once at the composition root and injected into every module.

use std::sync::Arc;

use crate::errors::Outcome;
use crate::services::vault::VaultTrait;
use crate::services::vault::global::VaultService;
use sea_orm::DatabaseConnection;

use crate::config::traits::{ConnectionConfigTrait, EventsConfigTrait, OauthConfigTrait};
use crate::config::types::{CommonConfig, EventNodeIdentity, EventsConfig};
use crate::services::token_validator::OauthTokenValidatorTrait;
use crate::telemetry::Telemetry;

/// One vault, one DB pool, one token validator and event pipeline settings for the whole process.
#[derive(Clone)]
pub struct RootContext {
    pub vault: Arc<VaultService>,
    pub db: DatabaseConnection,
    pub validator: Arc<dyn OauthTokenValidatorTrait>,
    pub events_config: EventsConfig,
    pub node_identity: Arc<EventNodeIdentity>,
}

impl RootContext {
    /// Opens the database through the vault, validates event config and builds the validator.
    pub async fn connect(common: &CommonConfig, vault: Arc<VaultService>) -> Outcome<Self> {
        common.events().validate()?;
        let db = vault.get_db_connection(common).await?;
        let events_config = common.events().clone();
        let default_service = Telemetry::service_name().unwrap_or_else(|| "ymir-agent".to_string());
        let node_identity = Arc::new(EventNodeIdentity::from_config(&events_config, &default_service));
        Ok(Self {
            validator: common.oauth().validator(common.connection().is_prod)?,
            vault,
            db,
            events_config,
            node_identity,
        })
    }

    /// Overrides the logical service name when config did not pin one explicitly.
    pub fn with_default_service(self, service: &str) -> Self {
        self.node_identity.set_default_service(service);
        self
    }

    /// Builds a test-only context with default event configuration.
    pub fn for_test(
        vault: Arc<VaultService>,
        db: DatabaseConnection,
        validator: Arc<dyn OauthTokenValidatorTrait>,
    ) -> Self {
        let events_config = EventsConfig::default();
        let node_identity = Arc::new(EventNodeIdentity::from_config(&events_config, "test-agent"));
        Self {
            vault,
            db,
            validator,
            events_config,
            node_identity,
        }
    }
}
