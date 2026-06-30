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

use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::FafnirConfig;
use crate::capabilities::Did;
use crate::config::traits::{DidConfigTrait, WalletConfigTrait};
use crate::config::types::{DidConfig, HostType};
use crate::data::entities::wallet::{did, key, vc};
use crate::errors::{BadFormat, Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::services::vault::{VaultService, VaultTrait};
use crate::services::wallet::WalletTrait;
use crate::types::dids::{DidBuilder, DidDocument, DidService};
use crate::types::http::HttpBody;
use crate::types::secrets::PemHelper;
use crate::types::wallet::{DidSearch, Identity, KeyRef, OidcUri, WalletInfo};
use crate::utils::{ResponseExt, expect_from_env, http_client, json_headers};

use async_trait::async_trait;
use reqwest::Response;
use serde::de::DeserializeOwned;
use tracing::info;

/// Wallet implementation backed by an external Fafnir wallet instance.
///
/// This service acts as a thin HTTP client over the wallet API,
/// maintaining a local identity cache that is refreshed both on `link()`
/// and after any mutation that may change the active default DID.
pub struct FafnirService {
    config: FafnirConfig,
    identity: Arc<RwLock<Identity>>,
    services: Vec<DidService>,
}

impl FafnirService {
    /// Creates a new Fafnir wallet client and initializes the local identity cache.
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
        let default = Self::fetch::<did::Model>(&self.config, "dids", "default").await?;
        self.replace_identity_from(&default).await
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
        let identity = self.identity.read().await;
        Ok(identity.did().clone())
    }

    async fn get_did_doc(&self) -> Outcome<DidDocument> {
        let identity = self.identity.read().await;
        Ok(identity.did_doc().clone())
    }

    fn get_identity(&self) -> Arc<RwLock<Identity>> {
        self.identity.clone()
    }

    // ===== STORAGE (READ ONLY) ===================================================================
    async fn retrieve_did(&self, search: DidSearch) -> Outcome<did::Model> {
        let id = self.resolve_to_id(&search).await?;
        Self::fetch::<did::Model>(&self.config, "dids", &id).await
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

    async fn register_key(&self, plan: key::Plan) -> Outcome<key::Model> {
        let url = format!(
            "{}/keys/new",
            self.config.get_wallet_api_url(HostType::Http)
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::json(&plan)?)
            .await?;

        Self::parse_res_or_fail(res, &url, "POST").await
    }

    async fn register_did(&self, mut plan: did::Plan) -> Outcome<did::Model> {
        if plan.service.is_none() && !self.services.is_empty() {
            plan.service = Some(self.services.clone());
        }
        let url = format!(
            "{}/dids/new",
            self.config.get_wallet_api_url(HostType::Http)
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::json(&plan)?)
            .await?;

        let model: did::Model = Self::parse_res_or_fail(res, &url, "POST").await?;
        self.maybe_update_identity(&model).await?;
        Ok(model)
    }

    async fn store_vc(&self, plan: vc::Plan) -> Outcome<vc::Model> {
        let url = format!(
            "{}/vcs/store",
            self.config.get_wallet_api_url(HostType::Http)
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::json(&plan)?)
            .await?;

        Self::parse_res_or_fail(res, &url, "POST").await
    }

    async fn set_default_did(&self, search: DidSearch) -> Outcome<did::Model> {
        info!("FafnirService: set_default_did");
        let id = self.resolve_to_id(&search).await?;
        let url = format!(
            "{}/dids/default/{}",
            self.config.get_wallet_api_url(HostType::Http),
            id
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::None)
            .await?;

        let model: did::Model = Self::parse_res_or_fail(res, &url, "POST").await?;
        self.maybe_update_identity(&model).await?;
        Ok(model)
    }

    // ===== DID-KEY MANAGEMENT ====================================================================

    async fn add_key_to_did(&self, search: DidSearch, key_id: String) -> Outcome<did::Model> {
        let id = self.resolve_to_id(&search).await?;
        let url = format!(
            "{}/dids/{}/key/{}",
            self.config.get_wallet_api_url(HostType::Http),
            id,
            key_id
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::None)
            .await?;

        let model: did::Model = Self::parse_res_or_fail(res, &url, "POST").await?;
        self.maybe_update_identity(&model).await?;
        Ok(model)
    }

    async fn remove_key_from_did(
        &self,
        search: DidSearch,
        key_id: String,
    ) -> Outcome<did::Model> {
        let id = self.resolve_to_id(&search).await?;
        let url = format!(
            "{}/dids/{}/key/{}",
            self.config.get_wallet_api_url(HostType::Http),
            id,
            key_id
        );
        let res = http_client()
            .delete(&url, Some(json_headers()), HttpBody::None)
            .await?;

        let model: did::Model = Self::parse_res_or_fail(res, &url, "DELETE").await?;
        self.maybe_update_identity(&model).await?;
        Ok(model)
    }

    async fn set_default_key(&self, search: DidSearch, key_id: String) -> Outcome<did::Model> {
        let id = self.resolve_to_id(&search).await?;
        let url = format!(
            "{}/dids/{}/key/default/{}",
            self.config.get_wallet_api_url(HostType::Http),
            id,
            key_id
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::None)
            .await?;

        let model: did::Model = Self::parse_res_or_fail(res, &url, "POST").await?;
        self.maybe_update_identity(&model).await?;
        Ok(model)
    }

    // ===== DELETE OPERATIONS =====================================================================

    async fn delete_key(&self, id: &str) -> Outcome<()> {
        self.delete("keys", id).await
    }

    async fn delete_did(&self, search: DidSearch) -> Outcome<()> {
        // Resolve to the actual DID string so we can compare against the active identity.
        let target_did = match &search {
            DidSearch::Did(d) => d.clone(),
            DidSearch::Id(id) => {
                Self::fetch::<did::Model>(&self.config, "dids", id)
                    .await?
                    .did
            }
        };

        let active = self.identity.read().await.did().id().to_string();
        if active == target_did {
            return Err(Errors::format(
                BadFormat::Received,
                "Refusing to delete the active identity's DID. Switch the default first.",
                None,
            ));
        }

        // Re-use the path-id endpoint regardless of how the caller addressed the DID.
        let id = self.resolve_to_id(&search).await?;
        self.delete("dids", &id).await
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

    /// Returns the wallet internal id corresponding to the search.
    ///
    /// If the search already carries an internal id, returns it as-is.
    /// If it carries a DID string, queries the wallet and looks the entry up.
    async fn resolve_to_id(&self, search: &DidSearch) -> Outcome<String> {
        match search {
            DidSearch::Id(id) => Ok(id.clone()),
            DidSearch::Did(did) => {
                let all = self.retrieve_all_dids().await?;
                all.into_iter()
                    .find(|d| d.did == *did)
                    .map(|d| d.id)
                    .ok_or_else(|| {
                        Errors::missing_resource(did, "DID not stored in wallet", None)
                    })
            }
        }
    }

    /// If the model represents the new default DID, replace the cached identity.
    async fn maybe_update_identity(&self, model: &did::Model) -> Outcome<()> {
        if model.r#default {
            self.replace_identity_from(model).await?;
        }
        Ok(())
    }

    /// Unconditionally replace the cached identity from the model.
    async fn replace_identity_from(&self, model: &did::Model) -> Outcome<()> {
        let did = Did::parse(&model.did)?;
        let new = Identity::new(did, model.did_document.clone(), model.default_key.clone());
        let mut guard = self.identity.write().await;
        *guard = new;
        Ok(())
    }
}
