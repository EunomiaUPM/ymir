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

use std::sync::{Arc, RwLock};

use super::config::FafnirConfig;
use crate::capabilities::Did;
use crate::config::traits::{DidConfigTrait, HostsConfigTrait, WalletConfigTrait};
use crate::config::types::{DidConfig, HostType};
use crate::errors::{Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::services::vault::{VaultService, VaultTrait};
use crate::services::wallet::WalletTrait;
use crate::types::dids::{DidBuilder, DidDocument, DidService};
use crate::types::http::HttpBody;
use crate::types::secrets::{PemHelper, StringHelper};
// use crate::types::wallet::waltid::{DidsInfo, OidcUri};
use crate::types::wallet::{Identity, KeyRef, WalletInfo};
use crate::utils::{ResponseExt, expect_from_env, http_client, json_headers};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use tracing::info;
use crate::data::entities::shared::participant;
use crate::data::entities::wallet::{did, key, vc};
use crate::types::participants::ParticipantType;
use crate::types::wallet::waltid::OidcUri;

pub struct FafnirService {
    config: FafnirConfig,
    participant_type: ParticipantType,
    identity: RwLock<Arc<Identity>>,
    services: Vec<DidService>,
}

impl FafnirService {
    pub async fn new(
        config: FafnirConfig,
        vault: Arc<VaultService>,
        services: Vec<DidService>,
        participant_type: ParticipantType,
    ) -> Outcome<Self> {
        let (did_doc, keys) = Self::bootstrap(&config, vault, &services).await?;
        let did = Did::parse(&did_doc.id)?;
        let identity = Identity::new(did, did_doc, keys);
        Ok(Self {
            config,
            identity: RwLock::new(Arc::new(identity)),
            participant_type,
            services,
        })
    }
}

impl FafnirService {
    async fn bootstrap(
        config: &FafnirConfig,
        vault: Arc<VaultService>,
        services: &[DidService],
    ) -> Outcome<(DidDocument, KeyRef)> {
        // =====
        if let Ok(base) = Self::fetch::<did::Model>(config, "dids", "default").await {
            return Ok((base.did_document, base.default_key));
        }

        // ===== REGISTER KEY ======================================================================
        let priv_vault_path = expect_from_env("VAULT_APP_PRIV_KEY");
        let key_data: PemHelper = vault.read(None, &priv_vault_path).await?;

        let key_req = key::Plan {
            id: priv_vault_path,
            r#default: true,
            alias: "base".to_string(),
            pem: key_data.pem().to_string(),
        };

        let key_url = format!("{}/keys/new", config.get_wallet_api_url(HostType::Http));

        let res = http_client()
            .post(&key_url, Some(json_headers()), HttpBody::json(&key_req)?)
            .await?;

        let key_model: key::Model = if !res.status().is_success() {
            res.parse_json().await?
        } else {
            return Err(Errors::wallet(
                key_url,
                "POST",
                Some(res.status()),
                "Errors saving key on wallet",
                None,
            ));
        };

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

        let did_model: did::Model = if res.status().is_success() {
            res.parse_json().await?
        } else {
            return Err(Errors::wallet(
                did_url,
                "POST",
                Some(res.status()),
                "Errors saving did on wallet",
                None,
            ));
        };

        // ===== SET DEFAULT DID ===================================================================
        let url = format!(
            "{}/dids/{}/default",
            config.get_wallet_api_url(HostType::Http),
            did_model.id
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::None)
            .await?;

        if !res.status().is_success() {
            Err(Errors::wallet(
                url,
                "POST",
                Some(res.status()),
                "Errors saving default did",
                None,
            ))
        } else {
            Ok((did_model.did_document, did_model.default_key))
        }
    }
}

#[async_trait]
impl WalletTrait for FafnirService {
    // IT IS ALWAYS LINKED
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
            dids: dids.into_iter().map(did_entry_to_info).collect(),
        })
    }

    fn get_did(&self) -> Outcome<Did> {
        let iden = self
            .identity
            .read()
            .map_err(|_| Errors::read("identity", "identity lock poisoned", None))?;

        Ok(iden.did().clone())
    }

    fn get_did_doc(&self) -> Outcome<DidDocument> {
        let iden = self
            .identity
            .read()
            .map_err(|_| Errors::read("identity", "identity lock poisoned", None))?;
        Ok(iden.did_doc().clone())
    }

    fn get_identity(&self) -> Outcome<Arc<Identity>> {
        let iden = self
            .identity
            .read()
            .map_err(|_| Errors::read("identity", "identity lock poisoned", None))?;
        Ok(iden.clone())
    }

    // ════════════════════════ RETRIEVE FROM WALLET ════════════════════
    //

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

    // ════════════════════════ REGISTER STUFF ══════════════════════════

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
            default: false,
            alias: alias.unwrap_or("".to_string()),
            pem: pem_helper.pem().to_owned(),
        };

        let res = http_client()
            .post(&key_url, Some(json_headers()), HttpBody::json(&key_req)?)
            .await?;

        if res.status().is_success() {
            res.parse_json().await
        } else {
            Err(Errors::wallet(
                key_url,
                "POST",
                Some(res.status()),
                "Errors saving key on wallet",
                None,
            ))
        }
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
            alias: alias.unwrap_or("default".to_string()),
            builder: did_builder.clone(),
            keys: keys_id,
            service: services,
        };
        let res = http_client()
            .post(&did_url, Some(json_headers()), HttpBody::json(&did_req)?)
            .await?;

        if res.status().is_success() {
            res.parse_json().await
        } else {
            Err(Errors::wallet(
                did_url,
                "POST",
                Some(res.status()),
                "Errors saving did on wallet",
                None,
            ))
        }
    }

    async fn store_vc(&self, vc: String) -> Outcome<vc::Model> {
        let did_url = format!(
            "{}/vcs/store",
            self.config.get_wallet_api_url(HostType::Http)
        );

        // TODO
        let res = http_client()
            .post(&did_url, Some(json_headers()), HttpBody::Raw(vc))
            .await?;

        if res.status().is_success() {
            res.parse_json().await
        } else {
            Err(Errors::wallet(
                did_url,
                "POST",
                Some(res.status()),
                "Errors saving vc    on wallet",
                None,
            ))
        }
    }

    async fn set_default_did(&self, did: Did) -> Outcome<did::Model> {
        info!("FafnirService: set_default_did");
        todo!()
        let all = self.retrieve_all_dids().await?;
        let target = all
            .iter()
            .find(|d| d.did == did.id())
            .ok_or_else(|| Errors::missing_resource(did.id(), "DID not stored in wallet", None))?;
        self.set_default_did_with_id(target.id()).await
    }

    // ════════════════════════ DELETE ══════════════════════════════════

    async fn delete_key(&self, id: &str) -> Outcome<()> {
        self.delete("keys", id).await
    }
    async fn delete_did(&self, id: &str) -> Outcome<()> {
        self.delete("dids", id).await
    }
    async fn delete_vc(&self, id: &str) -> Outcome<()> {
        self.delete("vcs", id).await
    }

    // ════════════════════════ OIDC ═════════════════════════

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
        if res.status().is_success() {
            Ok(())
        } else {
            Err(Errors::wallet(
                url,
                "POST",
                Some(res.status()),
                "Errors processing oid4vci",
                None,
            ))
        }
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

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Errors::wallet(
                url,
                "POST",
                Some(res.status()),
                "Errors processing oid4vp",
                None,
            ))
        }
    }
}

impl FafnirService {
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
        if res.status().is_success() {
            res.parse_json().await
        } else {
            Err(Errors::wallet(
                url,
                "GET",
                Some(res.status()),
                format!("Error fetching {resource}/{id}"),
                None,
            ))
        }
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

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Errors::wallet(
                &url,
                "DELETE",
                Some(res.status()),
                format!("Error deleting {}/{}", resource, id),
                None,
            ))
        }
    }

    async fn set_default_did_with_id(&self, did: &str) -> Outcome<()> {
        let url = format!(
            "{}/dids/{}/default",
            self.config.get_wallet_api_url(HostType::Http),
            did
        );
        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::None)
            .await?;

        if !res.status().is_success() {
            Err(Errors::wallet(
                url,
                "POST",
                Some(res.status()),
                "Errors saving default did",
                None,
            ))
        } else {
            Ok(())
        }
    }
}

fn did_entry_to_info(did: did::Model) -> DidsInfo {
    let key_id = did
        .keys
        .iter()
        .next()
        .map(|k| k.internal().to_string())
        .unwrap_or_default();
    DidsInfo {
        document: serde_json::to_string(&did.did_document).unwrap_or_default(),
        did: did.did,
        alias: did.alias,
        key_id: key_id.to_string(),
        default: did.default,
        created_on: String::new(),
    }
}
