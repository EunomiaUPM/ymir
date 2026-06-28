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

use std::sync::{Arc};
use tokio::sync::{RwLock};

use super::config::FafnirConfig;
use crate::capabilities::Did;
use crate::config::traits::{DidConfigTrait, WalletConfigTrait};
use crate::config::types::{DidConfig, HostType};
use crate::data::entities::wallet::{did, key, vc};
use crate::errors::{Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::services::vault::{VaultService, VaultTrait};
use crate::services::wallet::WalletTrait;
use crate::types::dids::{DidBuilder, DidDocument, DidService};
use crate::types::http::HttpBody;
use crate::types::secrets::{PemHelper, StringHelper};
use crate::types::wallet::{Identity, KeyRef, OidcUri, WalletInfo};
use crate::utils::{ResponseExt, expect_from_env, http_client, json_headers};

use async_trait::async_trait;
use reqwest::Response;
use serde::de::DeserializeOwned;
use tracing::info;

/// Wallet implementation backed by an external Fafnir wallet instance.
///
/// This service acts as a thin HTTP client over the wallet API,
/// maintaining a local identity cache for frequently accessed
/// DID information.
pub struct FafnirService {
    config: FafnirConfig,
    identity: Arc<RwLock<Identity>>,
    services: Vec<DidService>,
}

impl FafnirService {
    /// Creates a new Fafnir wallet client and initializes
    /// the local identity cache.
    pub async fn new(
        config: FafnirConfig,
        vault: Arc<VaultService>,
        services: Vec<DidService>,
    ) -> Outcome<Self> {
        let (did_doc, keys) = Self::bootstrap(&config, vault, &services).await?;
        let did = Did::parse(&did_doc.id)?;
        let identity = Identity::new(did, did_doc, keys);
        Ok(Self {
            config,
            identity: Arc::new(RwLock::new(identity)),
            services,
        })
    }

    /// Initializes the wallet identity.
    ///
    /// If a default DID already exists in the wallet it is reused.
    /// Otherwise, a new key, DID and default identity are created.
    async fn bootstrap(
        config: &FafnirConfig,
        vault: Arc<VaultService>,
        services: &[DidService],
    ) -> Outcome<(DidDocument, KeyRef)> {
        // ===== IF DATA IS SAVED IN WALLET RETRIEVE ===============================================
        if let Ok(base) = Self::fetch::<did::Model>(config, "dids", "default").await {
            return Ok((base.did_document, base.default_key));
        }

        // ===== REGISTER KEY ======================================================================
        let priv_vault_path = expect_from_env("VAULT_APP_PRIV_KEY");
        let key_data: PemHelper = vault.read(None, &priv_vault_path).await?;

        let key_req = key::Plan {
            id: priv_vault_path,
            alias: "base".to_string(),
            pem: key_data.pem().to_string(),
        };

        let key_url = format!("{}/keys/new", config.get_wallet_api_url(HostType::Http));

        let res = http_client()
            .post(&key_url, Some(json_headers()), HttpBody::json(&key_req)?)
            .await?;

        let key_model: key::Model = Self::parse_res_or_fail(res, &key_url, "POST").await?;

        // ===== REGISTER DID ======================================================================
        let did_builder = match config.did_config() {
            DidConfig::Jwk => DidBuilder::new_jwk(key_data.pem()),
            DidConfig::Web { web_config } => DidBuilder::new_web(
                &web_config.domain,
                web_config.path.as_deref(),
                web_config.port.as_deref(),
            ),
            DidConfig::Other(did) => {
                return Err(Errors::not_impl(
                    format!("did type {did} not supported"),
                    None,
                ));
            }
        };

        let did_url = format!("{}/dids/new", config.get_wallet_api_url(HostType::Http));
        let services = if services.is_empty() {
            None
        } else {
            Some(services.to_vec())
        };
        let did_req = did::Plan {
            alias: "base".to_string(),
            builder: did_builder,
            keys: vec![key_model.id],
            service: services,
        };
        let res = http_client()
            .post(&did_url, Some(json_headers()), HttpBody::json(&did_req)?)
            .await?;

        let did_model: did::Model = Self::parse_res_or_fail(res, &did_url, "POST").await?;

        Ok((did_model.did_document, did_model.default_key))
    }
}

#[async_trait]
impl WalletTrait for FafnirService {
    // ===== CORE WALLET STATE =====================================================================
    async fn link(&self) -> Outcome<()> {
        Ok(())
    }

    async fn get_wallet(&self) -> Outcome<WalletInfo> {
        let dids = self.retrieve_all_dids().await?;

        Ok(WalletInfo {
            id: "fafnir-local".to_string(),
            name: "fafnir-wallet".to_string(),
            created_on: String::new(),
            added_on: String::new(),
            permission: "Administrator".to_string(),
            dids,
        })
    }

    async fn get_did(&self) -> Outcome<Did> {
        let identity = self
            .identity
            .read().await;

        Ok(identity.did().clone())
    }

    async fn get_did_doc(&self) -> Outcome<DidDocument> {
        let identity = self
            .identity
            .read().await;

        Ok(identity.did_doc().clone())
    }

    fn get_identity(&self) -> Arc<RwLock<Identity>> {
        self.identity.clone()
    }

    // ===== STORAGE (READ ONLY) ===================================================================
    async fn retrieve_did(&self, id: &str) -> Outcome<did::Model> {
        Self::fetch::<did::Model>(&self.config, "dids", id).await
    }
    async fn retrieve_default_did(&self) -> Outcome<did::Model> {
        Self::fetch::<did::Model>(&self.config, "dids", "default").await
    }
    async fn retrieve_all_dids(&self) -> Outcome<Vec<did::Model>> {
        Self::fetch::<Vec<did::Model>>(&self.config, "dids", "all").await
    }

    async fn retrieve_key(&self, id: &str) -> Outcome<key::Model> {
        Self::fetch::<key::Model>(&self.config, "keys", id).await
    }

    async fn retrieve_all_keys(&self) -> Outcome<Vec<key::Model>> {
        Self::fetch::<Vec<key::Model>>(&self.config, "keys", "all").await
    }

    async fn retrieve_vc(&self, id: &str) -> Outcome<vc::Model> {
        Self::fetch::<vc::Model>(&self.config, "vcs", id).await
    }

    async fn retrieve_all_vcs(&self) -> Outcome<Vec<vc::Model>> {
        Self::fetch::<Vec<vc::Model>>(&self.config, "vcs", "all").await
    }

    // ===== STORAGE (MUTATIONS) ===================================================================

    async fn register_key(
        &self,
        pem_helper: &PemHelper,
        alias: Option<String>,
    ) -> Outcome<key::Model> {
        let key_url = format!(
            "{}/keys/new",
            self.config.get_wallet_api_url(HostType::Http)
        );
        let id = uuid::Uuid::new_v4().to_string();
        let key_req = key::Plan {
            id: format!("crypto/keys/{id}"),
            alias: alias.unwrap_or("".to_string()),
            pem: pem_helper.pem().to_owned(),
        };

        let res = http_client()
            .post(&key_url, Some(json_headers()), HttpBody::json(&key_req)?)
            .await?;

        Self::parse_res_or_fail(res, &key_url, "POST").await
    }

    async fn register_did(
        &self,
        did_builder: &DidBuilder,
        keys_id: Vec<String>,
        alias: Option<String>,
    ) -> Outcome<did::Model> {
        let did_url = format!(
            "{}/dids/new",
            self.config.get_wallet_api_url(HostType::Http)
        );
        let services = if self.services.is_empty() {
            None
        } else {
            Some(self.services.clone())
        };
        let did_req = did::Plan {
            alias: alias.unwrap_or("".to_string()),
            builder: did_builder.clone(),
            keys: keys_id,
            service: services,
        };
        let res = http_client()
            .post(&did_url, Some(json_headers()), HttpBody::json(&did_req)?)
            .await?;

        Self::parse_res_or_fail(res, &did_url, "POST").await
    }

    async fn store_vc(&self, vc: String) -> Outcome<vc::Model> {
        let vc_url = format!(
            "{}/vcs/store",
            self.config.get_wallet_api_url(HostType::Http)
        );

        let vc = StringHelper::new(vc);
        let res = http_client()
            .post(&vc_url, Some(json_headers()), HttpBody::json(&vc)?)
            .await?;

        Self::parse_res_or_fail(res, &vc_url, "POST").await
    }

    async fn set_default_did(&self, did: Did) -> Outcome<did::Model> {
        info!("FafnirService: set_default_did");

        let all = self.retrieve_all_dids().await?;
        let target = all
            .iter()
            .find(|d| d.did == did.id())
            .ok_or_else(|| Errors::missing_resource(did.id(), "DID not stored in wallet", None))?;
        self.set_default_did_with_id(&target.id).await
    }

    // ===== DELETE OPERATIONS =====================================================================

    async fn delete_key(&self, id: &str) -> Outcome<()> {
        self.delete("keys", id).await
    }
    async fn delete_did(&self, id: &str) -> Outcome<()> {
        self.delete("dids", id).await
    }
    async fn delete_vc(&self, id: &str) -> Outcome<()> {
        self.delete("vcs", id).await
    }

    // ===== PROTOCOL HANDLING =====================================================================

    async fn process_oid4vci(&self, uri: &str) -> Outcome<()> {
        info!("FafnirService: process_oid4vci({})", uri);
        let url = format!("{}/oid4vci", self.config.get_wallet_api_url(HostType::Http));
        let res = http_client()
            .post(
                &url,
                Some(json_headers()),
                HttpBody::json(&OidcUri {
                    uri: uri.to_string(),
                })?,
            )
            .await?;

        Self::check_or_fail(res, &url, "POST")
    }

    async fn process_oid4vp(&self, uri: &str) -> Outcome<()> {
        info!("FafnirService: process_oid4vp({})", uri);
        let url = format!("{}/oid4vp", self.config.get_wallet_api_url(HostType::Http));
        let res = http_client()
            .post(
                &url,
                Some(json_headers()),
                HttpBody::json(&OidcUri {
                    uri: uri.to_string(),
                })?,
            )
            .await?;

        Self::check_or_fail(res, &url, "POST")
    }
}

// ===== INTERNAL HELPERS ==========================================================================
impl FafnirService {
    async fn parse_res_or_fail<T: DeserializeOwned>(
        res: Response,
        url: &str,
        method: &str,
    ) -> Outcome<T> {
        if res.status().is_success() {
            res.parse_json().await
        } else {
            Err(Errors::wallet(
                url,
                method,
                Some(res.status()),
                "unexpected http status",
                None,
            ))
        }
    }
    fn check_or_fail(res: Response, url: &str, method: &str) -> Outcome<()> {
        if res.status().is_success() {
            Ok(())
        } else {
            Err(Errors::wallet(
                url,
                method,
                Some(res.status()),
                "unexpected http status",
                None,
            ))
        }
    }

    async fn fetch<T: DeserializeOwned>(
        config: &FafnirConfig,
        resource: &str,
        id: &str,
    ) -> Outcome<T> {
        let url = format!(
            "{}/{}/{}",
            config.get_wallet_api_url(HostType::Http),
            resource,
            id
        );
        let res = http_client().get(&url, Some(json_headers())).await?;

        Self::parse_res_or_fail(res, &url, "GET").await
    }
    async fn delete(&self, resource: &str, id: &str) -> Outcome<()> {
        let url = format!(
            "{}/{}/{}",
            self.config.get_wallet_api_url(HostType::Http),
            resource,
            id
        );
        let res = http_client()
            .delete(&url, Some(json_headers()), HttpBody::None)
            .await?;

        Self::check_or_fail(res, &url, "DELETE")
    }

    async fn set_default_did_with_id(&self, did: &str) -> Outcome<did::Model> {
        let url = format!(
            "{}/dids/{}/default",
            self.config.get_wallet_api_url(HostType::Http),
            did
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::None)
            .await?;

        Self::parse_res_or_fail(res, &url, "POST").await
    }
}
