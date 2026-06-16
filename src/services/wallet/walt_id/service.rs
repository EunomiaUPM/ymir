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

use async_trait::async_trait;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use chrono::{DateTime, Utc};
use reqwest::{Response, Url};
use serde_json::Value;
use std::str::FromStr;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use urlencoding::decode;

use super::super::WalletTrait;
use super::config::WaltIdConfig;
use crate::capabilities::Did;
use crate::config::traits::{DidConfigTrait, HostsConfigTrait, WalletConfigTrait};
use crate::config::types::{DidConfig, HostType};
use crate::data::entities::{mates, minions};
use crate::errors::{BadFormat, Errors, MissingAction, Outcome};
use crate::services::client::ClientTrait;
use crate::services::vault::VaultTrait;
use crate::services::vault::global::VaultService;
use crate::types::dids::{DidBuilder, DidDocument, DidService, DidType};
use crate::types::http::HttpBody;
use crate::types::keys::{Alg, Crv, Kty};
use crate::types::secrets::{PemHelper, SemiWaltIdSecrets};
use crate::types::vcs::VPDef;
use crate::types::wallet::waltid::{
    AuthJwtClaims, CredentialOfferResponse, DidsInfo, KeyDefinition, MatchVCsRequest, MatchingVCs,
    RedirectResponse, WalletCredentials, WalletInfoResponse, WalletLoginResponse, WalletSession,
};
use crate::types::wallet::{DidModel, KeyModel, KeyRef, VcBodyType, VcModel};
use crate::types::wallet::{Identity, WalletInfo};
use crate::utils::{
    ParseHeaderExt, ResponseExt, decode_url_safe_no_pad, expect_from_env, http_client, json_headers,
};

pub struct WaltIdService {
    wallet_session: Arc<Mutex<WalletSession>>,
    key_data: Arc<Mutex<Vec<KeyDefinition>>>,
    services: Vec<DidService>,
    vault: Arc<VaultService>,
    config: WaltIdConfig,
    identity: RwLock<Option<Identity>>,
}

impl WaltIdService {
    pub async fn new(
        config: WaltIdConfig,
        vault: Arc<VaultService>,
        services: Vec<DidService>,
    ) -> Outcome<Self> {
        let service = WaltIdService {
            wallet_session: Arc::new(Mutex::new(WalletSession {
                account_id: None,
                token: None,
                token_exp: None,
                wallets: vec![],
            })),
            key_data: Arc::new(Mutex::new(Vec::new())),
            config,
            vault,
            services,
            identity: RwLock::new(None),
        };

        let fresh = service.register().await?;
        service.login().await?;
        service.retrieve_wallet_info().await?;
        service.retrieve_wallet_keys().await?;
        service.retrieve_wallet_dids().await?;
        if fresh {
            service.load_own_identity().await?;
        }
        service.cache_identity().await?;

        Ok(service)
    }
}

#[async_trait]
impl WalletTrait for WaltIdService {
    async fn link(&self) -> Outcome<(mates::NewModel, minions::NewModel)> {
        self.login().await?;
        let url = self.config.hosts().get_host(HostType::Http);
        Ok((self.get_self_mate(url.clone())?, self.get_self_minion(url)?))
    }

    async fn get_wallet(&self) -> Outcome<WalletInfo> {
        let wallet_session = self.wallet_session.lock().await;
        wallet_session.wallets.first().cloned().ok_or_else(|| {
            Errors::missing_action(
                MissingAction::Wallet,
                "There is no wallet to retrieve dids from",
                None,
            )
        })
    }

    fn get_did(&self) -> Outcome<Did> {
        let guard = self
            .identity
            .read()
            .map_err(|_| Errors::read("identity", "identity lock poisoned", None))?;
        guard.as_ref().map(|id| id.did().clone()).ok_or_else(|| {
            Errors::missing_action(MissingAction::Did, "wallet not linked yet", None)
        })
    }

    fn get_did_doc(&self) -> Outcome<DidDocument> {
        let guard = self
            .identity
            .read()
            .map_err(|_| Errors::read("identity", "identity lock poisoned", None))?;
        guard
            .as_ref()
            .map(|id| id.did_doc().clone())
            .ok_or_else(|| {
                Errors::missing_action(MissingAction::Did, "wallet not linked yet", None)
            })
    }

    fn get_identity(&self) -> Outcome<Arc<Identity>> {
        let guard = self
            .identity
            .read()
            .map_err(|_| Errors::read("identity", "identity lock poisoned", None))?;
        let iden = guard.as_ref().ok_or_else(|| Errors::missing_action(MissingAction::Onboarding, "wallet has not onboarded yet (crazy)", None))?;
        Ok(Arc::new(iden.clone()))
    }

    async fn retrieve_did(&self, id: &str) -> Outcome<DidModel> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/dids/{}", wallet.id, id);
        let res = self
            .request(
                "GET",
                &path,
                HttpBody::None,
                true,
                true,
                "Petition to retrieve did failed",
            )
            .await?;
        let info: DidsInfo = res.parse_json().await?;
        dids_info_to_did_model(info)
    }

    async fn retrieve_default_did(&self) -> Outcome<DidModel> {
        let wallet = self.get_wallet().await?;
        let info = wallet
            .dids
            .iter()
            .find(|d| d.default)
            .cloned()
            .ok_or_else(|| Errors::missing_action(MissingAction::Did, "No default did", None))?;
        dids_info_to_did_model(info)
    }

    async fn retrieve_all_dids(&self) -> Outcome<Vec<DidModel>> {
        let wallet = self.get_wallet().await?;
        wallet
            .dids
            .into_iter()
            .map(dids_info_to_did_model)
            .collect()
    }

    async fn retrieve_key(&self, id: &str) -> Outcome<KeyModel> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/keys/{}", wallet.id, id);
        let res = self
            .request(
                "GET",
                &path,
                HttpBody::None,
                true,
                true,
                "Petition to retrieve key failed",
            )
            .await?;
        let key: KeyDefinition = res.parse_json().await?;
        Ok(key_def_to_key_model(key))
    }

    async fn retrieve_all_keys(&self) -> Outcome<Vec<KeyModel>> {
        let keys = self.key_data.lock().await;
        Ok(keys.iter().cloned().map(key_def_to_key_model).collect())
    }

    async fn retrieve_vc(&self, id: &str) -> Outcome<VcModel> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/credentials/{}", wallet.id, id);
        let res = self
            .request(
                "GET",
                &path,
                HttpBody::None,
                true,
                true,
                "Petition to retrieve credential failed",
            )
            .await?;
        let wc: WalletCredentials = res.parse_json().await?;
        Ok(wc_to_vc(wc))
    }

    async fn retrieve_all_vcs(&self) -> Outcome<Vec<VcModel>> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/credentials?showDeleted=false", wallet.id);
        let res = self
            .request(
                "GET",
                &path,
                HttpBody::None,
                true,
                true,
                "Petition to retrieve credentials failed",
            )
            .await?;
        let creds: Vec<WalletCredentials> = res.parse_json().await?;
        Ok(creds.into_iter().map(wc_to_vc).collect())
    }

    async fn register_key(
        &self,
        pem_helper: &PemHelper,
        alias: Option<String>,
    ) -> Outcome<KeyModel> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/keys/import", wallet.id);
        self.request(
            "POST",
            &path,
            HttpBody::Raw(pem_helper.pem().to_string()),
            true,
            false,
            "Petition to register key failed",
        )
            .await?;
        self.retrieve_wallet_keys().await?;
        let keys = self.key_data.lock().await;
        let last = keys.last().cloned().ok_or_else(|| {
            Errors::missing_action(MissingAction::Key, "Key register failed", None)
        })?;
        Ok(KeyModel {
            id: last.key_id.id,
            alias: alias.unwrap_or_default(),
            kty: pem_helper.kty().clone(),
            crv: pem_helper.crv().cloned(),
            pem: pem_helper.pem().to_string(),
        })
    }

    async fn register_did(
        &self,
        _did_builder: &DidBuilder,
        _keys_id: Vec<String>,
        alias: Option<String>,
    ) -> Outcome<DidModel> {
        let res = match self.config.did_config() {
            DidConfig::Web { web_config } => {
                self.reg_did_web(&web_config.domain, web_config.path.as_deref().unwrap_or(""))
                    .await?
            }
            DidConfig::Jwk => self.reg_did_jwk().await?,
            DidConfig::Other(s) => {
                return Err(Errors::not_impl(
                    format!("did type {s} not supported"),
                    None,
                ));
            }
        };
        if !res.status().is_success() {
            return Err(Errors::wallet(
                "register_did",
                "POST",
                Some(res.status()),
                "Register did failed",
                None,
            ));
        }
        let did_str = res.parse_text().await?;
        let did_str = did_str.trim().to_string();
        self.retrieve_wallet_dids().await?;
        let wallet = self.get_wallet().await?;
        let info = wallet
            .dids
            .iter()
            .find(|d| d.did == did_str)
            .cloned()
            .or_else(|| wallet.dids.last().cloned())
            .ok_or_else(|| {
                Errors::missing_action(MissingAction::Did, "Just-registered did not found", None)
            })?;
        let mut entry = dids_info_to_did_model(info)?;
        if let Some(a) = alias {
            entry.alias = a;
        }
        Ok(entry)
    }

    async fn set_default_did(&self, did: Did) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/dids/default?did={}", wallet.id, did.id());
        self.request(
            "POST",
            &path,
            HttpBody::None,
            true,
            true,
            "Petition to set did as default failed",
        )
            .await?;
        Ok(())
    }

    async fn delete_key(&self, id: &str) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/keys/{}", wallet.id, id);
        self.request(
            "DELETE",
            &path,
            HttpBody::None,
            true,
            false,
            "Petition to delete key failed",
        )
            .await?;
        Ok(())
    }

    async fn delete_did(&self, id: &str) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/dids/{}", wallet.id, id);
        self.request(
            "DELETE",
            &path,
            HttpBody::None,
            true,
            false,
            "Petition to delete did failed",
        )
            .await?;
        Ok(())
    }

    async fn delete_vc(&self, id: &str) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/credentials/{}", wallet.id, id);
        self.request(
            "DELETE",
            &path,
            HttpBody::None,
            true,
            false,
            "Petition to delete vc failed",
        )
            .await?;
        Ok(())
    }

    async fn process_oid4vci(&self, uri: &str) -> Outcome<()> {
        let cred_offer = self.resolve_credential_offer(uri).await?;
        let _issuer_metadata = self.resolve_credential_issuer(&cred_offer).await?;
        self.use_offer_req(uri, &cred_offer).await
    }

    async fn process_oid4vp(&self, uri: &str) -> Outcome<()> {
        let vpd = self.get_vpd(uri).await?;
        let vcs_id = self.get_matching_vcs(&vpd).await?;
        self.present_vp(uri, vcs_id).await?;
        Ok(())
    }
}

impl WaltIdService {
    async fn request(
        &self,
        method: &str,
        path: &str,
        body: HttpBody,
        use_auth: bool,
        is_json: bool,
        error_msg: &str,
    ) -> Outcome<Response> {
        let url = format!(
            "{}/wallet-api{}",
            self.config.get_wallet_api_url(HostType::Http),
            path
        );
        let mut headers = if is_json {
            json_headers()
        } else {
            let mut h = HeaderMap::new();
            h.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
            h.insert(ACCEPT, HeaderValue::from_static("application/json"));
            h
        };

        if use_auth {
            let token = self.get_token().await?;
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);
        }

        let res = match method {
            "GET" => http_client().get(&url, Some(headers)).await?,
            "POST" => http_client().post(&url, Some(headers), body).await?,
            "DELETE" => http_client().delete(&url, Some(headers), body).await?,
            _ => return Err(Errors::not_impl(format!("Method {}", method), None)),
        };

        if res.status().is_success() {
            Ok(res)
        } else {
            Err(Errors::wallet(
                &url,
                method,
                Some(res.status()),
                error_msg,
                None,
            ))
        }
    }

    async fn get_token(&self) -> Outcome<String> {
        let wallet_session = self.wallet_session.lock().await;
        wallet_session.token.as_ref().cloned().ok_or_else(|| {
            Errors::missing_action(
                MissingAction::Token,
                "There is no token available for use",
                None,
            )
        })
    }

    async fn get_key(&self) -> Outcome<KeyDefinition> {
        let key_data = self.key_data.lock().await;
        key_data.first().cloned().ok_or_else(|| {
            Errors::missing_action(MissingAction::Key, "No key found in wallet", None)
        })
    }

    async fn first_wallet_mut(&self) -> Outcome<tokio::sync::MutexGuard<'_, WalletSession>> {
        let wallet_session = self.wallet_session.lock().await;
        if wallet_session.wallets.is_empty() {
            Err(Errors::missing_action(
                MissingAction::Wallet,
                "There is no wallet available",
                None,
            ))
        } else {
            Ok(wallet_session)
        }
    }

    async fn register(&self) -> Outcome<bool> {
        info!("Registering in web wallet");
        let url = format!(
            "{}/wallet-api/auth/register",
            self.config.get_wallet_api_url(HostType::Http)
        );
        let db_path = expect_from_env("VAULT_APP_WALLET");
        let body = self.vault.read(None, &db_path).await?;

        let res = http_client()
            .post(&url, Some(json_headers()), HttpBody::Json(body))
            .await?;

        if res.status().is_success() {
            info!("Wallet account registration successful");
            Ok(true)
        } else if res.status().as_u16() == 409 {
            warn!("Wallet account has already registered");
            Ok(false)
        } else {
            Err(Errors::wallet(
                &url,
                "POST",
                Some(res.status()),
                "Petition to register Wallet failed",
                None,
            ))
        }
    }

    async fn login(&self) -> Outcome<()> {
        info!("Login into web wallet");

        let db_path = expect_from_env("VAULT_APP_WALLET");
        let body: SemiWaltIdSecrets = self.vault.read(None, &db_path).await?;

        let res = self
            .request(
                "POST",
                "/auth/login",
                HttpBody::json(&body)?,
                false,
                true,
                "Petition to login into Wallet failed",
            )
            .await?;

        let json_res: WalletLoginResponse = res.parse_json().await?;

        let mut wallet_session = self.wallet_session.lock().await;
        wallet_session.account_id = Some(json_res.id);

        let jwt = json_res.token;
        let jwt_parts: Vec<&str> = jwt.split('.').collect();
        if jwt_parts.len() != 3 {
            return Err(Errors::format(
                BadFormat::Sent,
                "The jwt does not have the correct format",
                None,
            ));
        }

        let decoded = decode_url_safe_no_pad(jwt_parts[1])?;
        let claims: AuthJwtClaims = serde_json::from_slice(&decoded)?;
        wallet_session.token_exp = Some(claims.exp);
        wallet_session.token = Some(jwt);

        info!("Login data saved successfully");
        Ok(())
    }

    async fn retrieve_wallet_info(&self) -> Outcome<()> {
        let res = self
            .request(
                "GET",
                "/wallet/accounts/wallets",
                HttpBody::None,
                true,
                true,
                "Petition to retrieve Wallet information failed",
            )
            .await?;

        let weird_wallets: WalletInfoResponse = res.parse_json().await?;
        let mut wallets = Vec::<WalletInfo>::new();
        for wallet in weird_wallets.wallets {
            let wallet = wallet.to_normal();
            if !wallets.contains(&wallet) {
                wallets.push(wallet);
            }
        }
        let mut wallet_session = self.wallet_session.lock().await;
        for wallet in wallets {
            if !wallet_session.wallets.contains(&wallet) {
                wallet_session.wallets.push(wallet);
            }
        }
        Ok(())
    }

    async fn retrieve_wallet_keys(&self) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/keys", wallet.id);

        let res = self
            .request(
                "GET",
                &path,
                HttpBody::None,
                true,
                false,
                "Petition to retrieve keys failed",
            )
            .await?;

        let keys: Vec<KeyDefinition> = res.parse_json().await?;
        let mut key_data = self.key_data.lock().await;
        for key in keys {
            if !key_data.contains(&key) {
                key_data.push(key);
            }
        }
        Ok(())
    }

    async fn retrieve_wallet_dids(&self) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/dids", wallet.id);

        let res = self
            .request(
                "GET",
                &path,
                HttpBody::None,
                true,
                true,
                "Petition to retrieve Wallet DIDs failed",
            )
            .await?;

        let dids: Vec<DidsInfo> = res.parse_json().await?;

        let mut wallet_session = self.first_wallet_mut().await?;
        let wallet = wallet_session.wallets.first_mut().unwrap();
        for did in dids {
            if !wallet.dids.contains(&did) {
                wallet.dids.push(did)
            }
        }
        Ok(())
    }

    async fn register_key_internal(&self) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let priv_key_path = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: PemHelper = self.vault.read(None, &priv_key_path).await?;

        let path = format!("/wallet/{}/keys/import", wallet.id);

        self.request(
            "POST",
            &path,
            HttpBody::Raw(priv_key.pem().to_string()),
            true,
            false,
            "Petition to register key failed",
        )
            .await?;
        Ok(())
    }

    async fn register_did_internal(&self) -> Outcome<Option<String>> {
        let res = match self.config.did_config() {
            DidConfig::Web { web_config } => {
                self.reg_did_web(&web_config.domain, web_config.path.as_deref().unwrap_or(""))
                    .await?
            }
            DidConfig::Jwk => self.reg_did_jwk().await?,
            DidConfig::Other(method) => {
                return Err(Errors::not_impl(
                    format!("did type {method} not supported"),
                    None,
                ));
            }
        };
        if res.status().is_success() {
            let res = res.parse_text().await?;
            debug!("{:#?}", res);
            Ok(Some(res))
        } else if res.status().as_u16() == 409 {
            warn!("Did already exists");
            Ok(None)
        } else {
            Err(Errors::wallet(
                "http://register_did_in_wallet",
                "POST",
                Some(res.status()),
                "Petition to register did failed",
                None,
            ))
        }
    }

    async fn reg_did_jwk(&self) -> Outcome<Response> {
        let wallet = self.get_wallet().await?;
        let key_data = self.get_key().await?;

        let path = format!(
            "/wallet/{}/dids/create/jwk?keyId={}&alias=jwk",
            wallet.id, key_data.key_id.id
        );

        self.request(
            "POST",
            &path,
            HttpBody::None,
            true,
            true,
            "Petition to register did failed",
        )
            .await
    }

    async fn reg_did_web(&self, domain: &str, did_path: &str) -> Outcome<Response> {
        let wallet = self.get_wallet().await?;
        let key_data = self.get_key().await?;

        let path = format!(
            "/wallet/{}/dids/create/web?keyId={}&alias=web&domain={}&path={}",
            wallet.id, &key_data.key_id.id, domain, did_path
        );

        self.request(
            "POST",
            &path,
            HttpBody::None,
            true,
            true,
            "Petition to register did failed",
        )
            .await
    }

    async fn set_default_did_internal(&self, did: Option<&str>) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let did = match did {
            Some(did) => did.to_string(),
            None => {
                let wallet = self.get_wallet().await?;
                wallet.dids.first().map(|d| d.did.clone()).ok_or_else(|| {
                    Errors::missing_action(MissingAction::Did, "No DIDs found in wallet", None)
                })?
            }
        };

        let path = format!("/wallet/{}/dids/default?did={}", wallet.id, did);

        self.request(
            "POST",
            &path,
            HttpBody::None,
            true,
            true,
            "Petition to set did as default failed",
        )
            .await?;
        Ok(())
    }

    async fn load_own_identity(&self) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        if let Some(did_info) = wallet.dids.first().cloned() {
            self.delete_did(&did_info.did).await?;
        }
        if let Ok(key_data) = self.get_key().await {
            self.delete_key(&key_data.key_id.id).await?;
        }

        {
            let mut session = self.wallet_session.lock().await;
            if let Some(w) = session.wallets.first_mut() {
                w.dids.clear();
            }
        }
        self.key_data.lock().await.clear();

        self.register_key_internal().await?;
        self.retrieve_wallet_keys().await?;
        let did = self.register_did_internal().await?;
        self.set_default_did_internal(did.as_deref()).await?;
        self.retrieve_wallet_dids().await?;
        Ok(())
    }

    async fn cache_identity(&self) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let info = wallet.dids.first().ok_or_else(|| {
            Errors::missing_action(MissingAction::Did, "No dids in wallet to cache", None)
        })?;

        let did_doc: DidDocument = serde_json::from_str(&info.document)?;
        let did_doc = did_doc.add_services(self.services.clone());
        let did = Did::parse(&info.did)?;

        let mut guard = self
            .identity
            .write()
            .map_err(|_| Errors::read("identity", "identity lock poisoned", None))?;
        *guard = Some(Identity::new(did, did_doc, vec![]));
        Ok(())
    }

    async fn resolve_credential_offer(&self, uri: &str) -> Outcome<CredentialOfferResponse> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/exchange/resolveCredentialOffer", wallet.id);

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/plain;charset=UTF-8"),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        let token = self.get_token().await?;
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let url = format!(
            "{}/wallet-api{}",
            self.config.get_wallet_api_url(HostType::Http),
            path
        );
        let res = http_client()
            .post(&url, Some(headers), HttpBody::Raw(uri.to_string()))
            .await?;

        if res.status().is_success() {
            let data: CredentialOfferResponse = res.parse_json().await?;
            Ok(data)
        } else {
            Err(Errors::wallet(
                &url,
                "POST",
                Some(res.status()),
                "Petition to resolve credential offer failed",
                None,
            ))
        }
    }

    async fn resolve_credential_issuer(
        &self,
        cred_offer: &CredentialOfferResponse,
    ) -> Outcome<Value> {
        let wallet = self.get_wallet().await?;
        let path = format!(
            "/wallet/{}/exchange/resolveIssuerOpenIDMetadata?issuer={}",
            wallet.id, cred_offer.credential_issuer
        );

        let res = self
            .request(
                "GET",
                &path,
                HttpBody::None,
                true,
                true,
                "Petition resolve credential issuer failed",
            )
            .await?;

        let data: Value = res.parse_json().await?;
        Ok(data)
    }

    async fn use_offer_req(&self, uri: &str, cred_offer: &CredentialOfferResponse) -> Outcome<()> {
        let wallet = self.get_wallet().await?;
        let did = self.get_did()?;

        let path = format!(
            "/wallet/{}/exchange/useOfferRequest?did={}&requireUserInput=false&pinOrTxCode={}",
            wallet.id,
            did.id(),
            cred_offer.grants.pre_authorized_code.pre_authorized_code
        );

        let res = self
            .request(
                "POST",
                &path,
                HttpBody::Raw(uri.to_string()),
                true,
                true,
                "Petition accept credential issuer failed",
            )
            .await?;

        let data: Value = res.parse_json().await?;
        debug!("{:#?}", data);
        Ok(())
    }

    async fn get_vpd(&self, uri: &str) -> Outcome<VPDef> {
        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/exchange/resolvePresentationRequest", wallet.id);

        let res = self
            .request(
                "POST",
                &path,
                HttpBody::Raw(uri.to_string()),
                true,
                false,
                "Error joining the exchange",
            )
            .await?;

        let vpd = res.parse_text().await?;
        self.parse_vpd(&vpd)
    }

    fn parse_vpd(&self, vpd_as_string: &str) -> Outcome<VPDef> {
        let url = Url::parse(
            decode(vpd_as_string)
                .map_err(|e| Errors::parse("Unable to decode vpd", Some(Box::new(e))))?
                .as_ref(),
        )
            .map_err(|e| Errors::parse("Unable to extract url from string", Some(Box::new(e))))?;

        let vpd_json = &url
            .query_pairs()
            .find(|(k, _)| k == "presentation_definition")
            .map(|(_, v)| v.into_owned())
            .ok_or_else(|| {
                Errors::format(
                    BadFormat::Received,
                    "Missing query parameter presentation_definition",
                    None,
                )
            })?;
        Ok(serde_json::from_str(&vpd_json)?)
    }

    async fn get_matching_vcs(&self, vpd: &VPDef) -> Outcome<Vec<String>> {
        let mut vcs_id = Vec::with_capacity(vpd.input_descriptors.len());
        for descriptor in &vpd.input_descriptors {
            let n_vpd = VPDef {
                id: "temporal_id".to_string(),
                input_descriptors: vec![descriptor.clone()],
            };
            let vcs = self.match_vc4vp(serde_json::to_value(&n_vpd)?).await?;
            let vc_id = vcs.first().map(|data| data.id.clone()).ok_or_else(|| {
                Errors::missing_action(
                    MissingAction::Credentials,
                    "There are no VCs that match the specified input descriptor",
                    None,
                )
            })?;
            vcs_id.push(vc_id);
        }
        Ok(vcs_id)
    }

    async fn match_vc4vp(&self, vp_def: Value) -> Outcome<Vec<MatchingVCs>> {
        let wallet = self.get_wallet().await?;
        let path = format!(
            "/wallet/{}/exchange/matchCredentialsForPresentationDefinition",
            wallet.id
        );

        let res = self
            .request(
                "POST",
                &path,
                HttpBody::Json(vp_def),
                true,
                true,
                "Petition to match credentials failed",
            )
            .await?;

        let vc_json: Vec<MatchingVCs> = res.parse_json().await?;
        Ok(vc_json)
    }

    async fn present_vp(&self, uri: &str, vcs_id: Vec<String>) -> Outcome<Option<String>> {
        let wallet = self.get_wallet().await?;
        let did = self.get_did()?;

        let path = format!("/wallet/{}/exchange/usePresentationRequest", wallet.id);

        let body = MatchVCsRequest {
            did: did.id().to_string(),
            presentation_request: uri.to_string(),
            selected_credentials: vcs_id,
        };

        let res = self
            .request(
                "POST",
                &path,
                HttpBody::json(&body)?,
                true,
                true,
                "Petition to present credentials failed",
            )
            .await?;

        match res.json::<Option<RedirectResponse>>().await {
            Ok(Some(data)) => Ok(Some(data.redirect_uri)),
            _ => Ok(None),
        }
    }
}

fn wc_to_vc(wc: WalletCredentials) -> VcModel {
    let added_on = DateTime::parse_from_rfc3339(&wc.added_on)
        .map(|d| d.with_timezone(&Utc))
        .ok();
    let r#type = if wc.format.contains("jwt") {
        VcBodyType::Jwt(wc.document)
    } else {
        match serde_json::from_str::<Value>(&wc.document) {
            Ok(v) => VcBodyType::Value(v),
            Err(_) => VcBodyType::Jwt(wc.document),
        }
    };
    VcModel {
        id: wc.id,
        r#type,
        parsed_document: wc.parsed_document,
        added_on,
    }
}

fn dids_info_to_did_model(d: DidsInfo) -> Outcome<DidModel> {
    let did_document: DidDocument = serde_json::from_str(&d.document)?;
    let did_type = if d.did.starts_with("did:web:") {
        DidType::Web
    } else if d.did.starts_with("did:jwk:") {
        DidType::Jwk
    } else {
        return Err(Errors::not_impl(
            format!("Did method {} not supported", d.did),
            None,
        ));
    };
    let keys = did_document
        .verification_method
        .iter()
        .map(|vm| KeyRef::new(d.key_id.clone(), vm.id.clone()))
        .collect::<Vec<KeyRef>>();
    Ok(DidModel {
        did_id: d.did.clone(),
        did: d.did,
        alias: d.alias,
        default: d.default,
        r#type: did_type,
        keys,
        did_document,
    })
}

fn key_def_to_key_model(k: KeyDefinition) -> KeyModel {
    let Ok(alg) = Alg::from_str(&k.algorithm);
    let (kty, crv) = match &alg {
        Alg::EdDsa => (Kty::Okp, Some(Crv::Ed25519)),
        Alg::Rs256 | Alg::Ps256 => (Kty::Rsa, None),
        Alg::Es256 => (Kty::Ec, Some(Crv::P256)),
        _ => (Kty::Other(String::new()), None),
    };
    KeyModel {
        id: k.key_id.id,
        alias: String::new(),
        kty,
        crv,
        pem: String::new(),
    }
}
