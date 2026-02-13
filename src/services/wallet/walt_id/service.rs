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

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue};
use reqwest::{Response, Url};
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use urlencoding::decode;

use super::super::WalletTrait;
use super::config::WaltIdConfig;
use crate::config::traits::{DidConfigTrait, HostsConfigTrait, WalletConfigTrait};
use crate::config::types::HostType;
use crate::data::entities::{mates, minions};
use crate::errors::{Errors, Outcome};
use crate::services::client::ClientTrait;
use crate::services::vault::VaultTrait;
use crate::services::vault::vault_rs::VaultService;
use crate::types::dids::did_type::DidType;
use crate::types::dids::dids_info::DidsInfo;
use crate::types::errors::{BadFormat, MissingAction};
use crate::types::http::Body;
use crate::types::jwt::AuthJwtClaims;
use crate::types::secrets::{SemiWalletSecrets, StringHelper};
use crate::types::wallet::{
    CredentialOfferResponse, KeyDefinition, MatchVCsRequest, MatchingVCs, OidcUri,
    RedirectResponse, Vpd, WalletCredentials, WalletInfo, WalletInfoResponse, WalletLoginResponse,
    WalletSession
};
use crate::utils::{
    ParseHeaderExt, decode_url_safe_no_pad, expect_from_env, get_query_param, parse_from_slice,
    parse_from_str, parse_json_resp, parse_text_resp, parse_to_value
};

pub struct WaltIdService {
    wallet_session: Arc<Mutex<WalletSession>>,
    key_data: Arc<Mutex<Vec<KeyDefinition>>>,
    client: Arc<dyn ClientTrait>,
    vault: Arc<VaultService>,
    config: WaltIdConfig
}

impl WaltIdService {
    pub fn new(
        config: WaltIdConfig,
        client: Arc<dyn ClientTrait>,
        vault: Arc<VaultService>
    ) -> WaltIdService {
        WaltIdService {
            wallet_session: Arc::new(Mutex::new(WalletSession {
                account_id: None,
                token: None,
                token_exp: None,
                wallets: vec![]
            })),
            key_data: Arc::new(Mutex::new(Vec::new())),
            config,
            client,
            vault
        }
    }
}

#[async_trait]
impl WalletTrait for WaltIdService {
    // BASIC -------------------------------------------------------------------------------------->
    async fn register(&self) -> Outcome<()> {
        info!("Registering in web wallet");
        let url = format!("{}/wallet-api/auth/register", self.config.get_wallet_api_url());
        let db_path = expect_from_env("VAULT_APP_WALLET");
        let body = self.vault.read(None, &db_path).await?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let res = self.client.post(&url, Some(headers), Body::Json(body)).await?;

        match res.status().as_u16() {
            201 => {
                info!("Wallet account registration successful");
            }
            409 => {
                warn!("Wallet account has already registered");
            }
            status => {
                return Err(Errors::wallet(
                    &url,
                    "POST",
                    Some(status),
                    "Petition to register wallet failed",
                    None
                ));
            }
        }
        Ok(())
    }

    async fn login(&self) -> Outcome<()> {
        info!("Login into web wallet");
        let url = format!("{}/wallet-api/auth/login", self.config.get_wallet_api_url());

        let db_path = expect_from_env("VAULT_APP_WALLET");
        let body: SemiWalletSecrets = self.vault.read(None, &db_path).await?;
        let body = parse_to_value(&body)?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let res = self.client.post(&url, Some(headers), Body::Json(body)).await?;

        match res.status().as_u16() {
            200 => {
                info!("Wallet login successful");

                let json_res: WalletLoginResponse = parse_json_resp(res).await?;

                let mut wallet_session = self.wallet_session.lock().await;
                wallet_session.account_id = Some(json_res.id);
                wallet_session.token = Some(json_res.token.clone());

                let jwt_parts: Vec<&str> = json_res.token.split('.').collect();
                if jwt_parts.len() != 3 {
                    return Err(Errors::format(
                        BadFormat::Sent,
                        "The jwt does not have the correct format",
                        None
                    ));
                }

                let decoded = decode_url_safe_no_pad(jwt_parts[1])?;
                let claims: AuthJwtClaims = parse_from_slice(&decoded)?;
                wallet_session.token_exp = Some(claims.exp);

                info!("Login data saved successfully");
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Petition to login into Wallet failed",
                None
            ))
        }
    }

    async fn logout(&self) -> Outcome<()> {
        info!("Login out of web wallet");
        let url = format!("{}/wallet-api/auth/logout", self.config.get_wallet_api_url());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let res = self.client.post(&url, Some(headers), Body::None).await?;

        match res.status().as_u16() {
            200 => {
                info!("Wallet logout successful");
                let mut wallet_session = self.wallet_session.lock().await;
                wallet_session.token = None;
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Petition to logout from Wallet failed",
                None
            ))
        }
    }

    async fn onboard(&self) -> Outcome<(mates::NewModel, minions::NewModel)> {
        info!("Onboarding into web wallet");

        self.register().await?;
        self.login().await?;
        self.retrieve_wallet_info().await?;
        self.retrieve_wallet_keys().await?;
        self.retrieve_wallet_dids().await?;

        let wallet = self.get_wallet().await?;
        let key_data = self.get_key().await?;
        let did_info = wallet.dids.first().cloned().ok_or_else(|| {
            Errors::missing_action(MissingAction::Did, "Something impossible happened", None)
        })?;

        self.delete_did(did_info).await?;
        self.delete_key(key_data).await?;

        self.register_key().await?;
        self.retrieve_wallet_keys().await?;

        self.register_did().await?;

        self.retrieve_wallet_info().await?;
        self.retrieve_wallet_dids().await?;
        self.set_default_did().await?;

        let did = self.get_did().await?;
        let mate = mates::NewModel {
            participant_id: did.clone(),
            participant_slug: "Myself".to_string(),
            participant_type: "Agent".to_string(),
            base_url: self.config.hosts().get_host(HostType::Http),
            token: None,
            is_me: true
        };
        let minion = minions::NewModel {
            participant_id: did,
            participant_slug: "Myself".to_string(),
            participant_type: "Authority".to_string(),
            base_url: Some(self.config.hosts().get_host(HostType::Http)),
            vc_uri: None,
            is_vc_issued: false,
            is_me: true
        };

        Ok((mate, minion))
    }

    async fn partial_onboard(&self) -> Outcome<()> {
        info!("Initializing partial onboarding");

        self.login().await?;
        self.retrieve_wallet_info().await?;
        self.retrieve_wallet_keys().await?;
        self.retrieve_wallet_dids().await?;

        info!("Initialization successful");
        Ok(())
    }

    // GET FROM STRUCT------------------------------------------------------------------------------------>
    async fn get_wallet(&self) -> Outcome<WalletInfo> {
        info!("Getting wallet data");
        let wallet_session = self.wallet_session.lock().await;

        wallet_session.wallets.first().map(Clone::clone).ok_or_else(|| {
            Errors::missing_action(
                MissingAction::Wallet,
                "There is no wallet to retrieve dids from",
                None
            )
        })
    }

    async fn first_wallet_mut(&self) -> Outcome<tokio::sync::MutexGuard<'_, WalletSession>> {
        let wallet_session = self.wallet_session.lock().await;

        if wallet_session.wallets.is_empty() {
            Err(Errors::missing_action(
                MissingAction::Wallet,
                "There is no wallet available",
                None
            ))
        } else {
            Ok(wallet_session)
        }
    }

    async fn get_did(&self) -> Outcome<String> {
        info!("Getting Did");
        let wallet = self.get_wallet().await?;

        wallet.dids.first().map(|data| data.did.clone()).ok_or_else(|| {
            Errors::missing_action(MissingAction::Did, "No DIDs found in wallet", None)
        })
    }

    async fn get_token(&self) -> Outcome<String> {
        info!("Getting token");

        let wallet_session = self.wallet_session.lock().await;
        wallet_session.token.clone().ok_or_else(|| {
            Errors::missing_action(
                MissingAction::Token,
                "There is no token available for use",
                None
            )
        })
    }

    async fn get_did_doc(&self) -> Outcome<Value> {
        info!("Getting Did Document");

        let wallet = self.get_wallet().await?;
        let did = wallet.dids.first().map(|data| data.document.clone()).ok_or_else(|| {
            Errors::missing_action(MissingAction::Did, "No dids found in wallet", None)
        })?;
        parse_to_value(&did)
    }

    async fn get_key(&self) -> Outcome<KeyDefinition> {
        info!("Getting key data");

        let key_data = self.key_data.lock().await;
        key_data.first().cloned().ok_or_else(|| {
            Errors::missing_action(MissingAction::Key, "No key found in wallet", None)
        })
    }

    // RETRIEVE FROM WALLET
    // ------------------------------------------------------------------------------->
    async fn retrieve_wallet_info(&self) -> Outcome<()> {
        info!("Retrieving wallet info from web wallet");
        let url = format!(
            "{}/wallet-api/wallet/accounts/wallets",
            self.config.get_wallet_api_url()
        );

        let token = self.get_token().await?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.get(&url, Some(headers)).await?;

        match res.status().as_u16() {
            200 => {
                let weird_wallets: WalletInfoResponse = parse_json_resp(res).await?;
                let weird_wallets = weird_wallets.wallets;
                let mut wallets = Vec::<WalletInfo>::new();
                for wallet in weird_wallets {
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
                info!("Wallet data loaded successfully");
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "GET",
                Some(status),
                "Petition to retrieve Wallet information failed",
                None
            ))
        }
    }

    async fn retrieve_wallet_keys(&self) -> Outcome<()> {
        info!("Retrieving keys from web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/keys",
            self.config.get_wallet_api_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.get(&url, Some(headers)).await?;

        match res.status().as_u16() {
            200 => {
                info!("Keys retrieved successfully");
                let res = parse_text_resp(res).await?;
                let keys: Vec<KeyDefinition> = parse_from_str(&res)?;
                let mut key_data = self.key_data.lock().await;
                for key in keys {
                    if !key_data.contains(&key) {
                        key_data.push(key);
                    }
                }
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "GET",
                Some(status),
                "Petition to retrieve keys failed",
                None
            ))
        }
    }

    async fn retrieve_wallet_dids(&self) -> Outcome<()> {
        info!("Retrieving dids from web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/dids",
            self.config.get_wallet_api_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.get(&url, Some(headers)).await?;

        match res.status().as_u16() {
            200 => {
                let dids: Vec<DidsInfo> = parse_json_resp(res).await?;

                let mut wallet_session = self.first_wallet_mut().await?;
                let wallet = wallet_session.wallets.first_mut().unwrap();

                for did in dids {
                    if !wallet.dids.contains(&did) {
                        wallet.dids.push(did)
                    }
                }

                info!("Wallet Dids data loaded successfully");
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "GET",
                Some(status),
                "Petition to retrieve Wallet DIDs failed",
                None
            ))
        }
    }

    async fn retrieve_wallet_credentials(&self) -> Outcome<Vec<WalletCredentials>> {
        info!("Retrieving credentials from web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/credentials",
            self.config.get_wallet_api_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.get(&url, Some(headers)).await?;

        match res.status().as_u16() {
            200 => {
                let res: Vec<WalletCredentials> = parse_json_resp(res).await?;
                info!("Wallet Credentials data loaded successfully");
                Ok(res)
            }
            status => Err(Errors::wallet(
                &url,
                "GET",
                Some(status),
                "Petition to retrieve Wallet Credentials failed",
                None
            ))
        }
    }

    // REGISTER STUFF IN WALLET
    // ----------------------------------------------------------------------------->
    async fn register_key(&self) -> Outcome<()> {
        info!("Registering key in web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let priv_key = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: StringHelper = self.vault.read(None, &priv_key).await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/keys/import",
            self.config.get_wallet_api_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res =
            self.client.post(&url, Some(headers), Body::Raw(priv_key.data().to_string())).await?;

        match res.status().as_u16() {
            201 => {
                info!("Key registered successfully");
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Petition to register key failed",
                None
            ))
        }
    }

    async fn register_did(&self) -> Outcome<()> {
        info!("Registering did in web wallet");

        let res = match self.config.get_did_type() {
            DidType::Web => self.reg_did_web().await?,
            DidType::Jwk => self.reg_did_jwk().await?,
            DidType::Other => return Err(Errors::not_impl("Other did type", None))
        };

        match res.status().as_u16() {
            200 => {
                info!("Did registered successfully");
                let res = parse_text_resp(res).await?;
                debug!("{:#?}", res);
                Ok(())
            }
            409 => {
                warn!("Did already exists");
                Ok(())
            }
            status => Err(Errors::wallet(
                "http://register_did_in_wallet",
                "POST",
                Some(status),
                "Petition to register key failed",
                None
            ))
        }
    }

    async fn reg_did_jwk(&self) -> Outcome<Response> {
        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let key_data = self.get_key().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/dids/create/jwk?keyId={}&alias=jwk",
            self.config.get_wallet_api_url(),
            &wallet.id,
            key_data.key_id.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        self.client.post(&url, Some(headers), Body::None).await
    }

    async fn reg_did_web(&self) -> Outcome<Response> {
        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let key_data = self.get_key().await?;

        let options =
            self.config.get_did_web_options().ok_or_else(|| Errors::not_active("did web", None))?;

        let path = match options.path.as_ref() {
            Some(path) => format!("&path={}", path),
            None => "".to_string()
        };

        let url = format!(
            "{}/wallet-api/wallet/{}/dids/create/web?keyId={}&alias=web&domain={}{}",
            self.config.get_wallet_api_url(),
            &wallet.id,
            &key_data.key_id.id,
            options.domain,
            path
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        self.client.post(&url, Some(headers), Body::None).await
    }

    async fn set_default_did(&self) -> Outcome<()> {
        info!("Setting default did in web wallet");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let did = self.get_did().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/dids/default?did={}",
            self.config.get_wallet_api_url(),
            &wallet.id,
            did
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.post(&url, Some(headers), Body::None).await?;

        match res.status().as_u16() {
            202 => {
                info!("Did has been set as default");
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Petition to set did as default failed",
                None
            ))
        }
    }

    // DELETE STUFF FROM WALLET
    // --------------------------------------------------------------------------->
    async fn delete_key(&self, key_id: KeyDefinition) -> Outcome<()> {
        info!("Deleting key in web wallet and from internal data");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/keys/{}",
            self.config.get_wallet_api_url(),
            &wallet.id,
            key_id.key_id.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.delete(&url, Some(headers), Body::None).await?;

        match res.status().as_u16() {
            202 => {
                info!("Key deleted successfully from web wallet");
                let mut keys_data = self.key_data.lock().await;
                keys_data.retain(|key| *key != key_id);
                info!("Key deleted successfully from internal data");
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "DELETE",
                Some(status),
                "Petition to delete key failed",
                None
            ))
        }
    }

    async fn delete_did(&self, did_info: DidsInfo) -> Outcome<()> {
        info!("Deleting did from web wallet and from internal data");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/dids/{}",
            self.config.get_wallet_api_url(),
            &wallet.id,
            did_info.did
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.delete(&url, Some(headers), Body::None).await?;

        match res.status().as_u16() {
            202 => {
                info!("Did deleted successfully from web wallet");
                let mut wallet_session = self.first_wallet_mut().await?;
                let wallet = wallet_session.wallets.first_mut().unwrap();

                wallet.dids.retain(|did| *did != did_info);
                info!("Did deleted successfully from internal data");
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "DELETE",
                Some(status),
                "Petition to delete key failed",
                None
            ))
        }
    }
    async fn resolve_credential_offer(
        &self,
        payload: &OidcUri
    ) -> Outcome<CredentialOfferResponse> {
        info!("Resolving credential offer");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/resolveCredentialOffer",
            self.config.get_wallet_api_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain;charset=UTF-8"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.post(&url, Some(headers), Body::Raw(payload.uri.clone())).await?;

        match res.status().as_u16() {
            200 => {
                let data: CredentialOfferResponse = parse_json_resp(res).await?;
                info!("Credential offer resolved successfully");
                Ok(data)
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Petition to resolve credential offer failed",
                None
            ))
        }
    }

    async fn resolve_credential_issuer(
        &self,
        cred_offer: &CredentialOfferResponse
    ) -> Outcome<Value> {
        info!("Resolving credential issuer metadata");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/resolveIssuerOpenIDMetadata?issuer={}",
            self.config.get_wallet_api_url(),
            &wallet.id,
            cred_offer.credential_issuer
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.get(&url, Some(headers)).await?;

        match res.status().as_u16() {
            200 => {
                let data: Value = parse_json_resp(res).await?;
                info!("Credential issuer resolved successfully");
                Ok(data)
            }
            status => Err(Errors::wallet(
                &url,
                "GET",
                Some(status),
                "Petition resolve credential issuer failed",
                None
            ))
        }
    }

    async fn use_offer_req(
        &self,
        payload: &OidcUri,
        cred_offer: &CredentialOfferResponse
    ) -> Outcome<()> {
        info!("Accepting credential");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let did = self.get_did().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/useOfferRequest?did={}&requireUserInput=false&pinOrTxCode={}",
            self.config.get_wallet_api_url(),
            &wallet.id,
            did,
            cred_offer.grants.pre_authorized_code.pre_authorized_code
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.post(&url, Some(headers), Body::Raw(payload.uri.clone())).await?;

        match res.status().as_u16() {
            200 => {
                let data: Value = parse_json_resp(res).await?;
                info!("Credential accepted successfully");
                debug!("{:#?}", data);
                Ok(())
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Petition accept credential issuer failed",
                None
            ))
        }
    }

    async fn get_vpd(&self, payload: &OidcUri) -> Outcome<Vpd> {
        info!("Joining exchange");

        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/resolvePresentationRequest",
            self.config.get_wallet_api_url(),
            &wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        headers.insert(ACCEPT, HeaderValue::from_static("text/plain"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.post(&url, Some(headers), Body::Raw(payload.uri.clone())).await?;

        match res.status().as_u16() {
            200 => {
                info!("Joined the exchange successful");
                let vpd = parse_text_resp(res).await?;
                let vpd = self.parse_vpd(&vpd)?;
                Ok(vpd)
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Error joining the exchange",
                None
            ))
        }
    }

    fn parse_vpd(&self, vpd_as_string: &str) -> Outcome<Vpd> {
        info!("Parsing Vpd");

        let url = Url::parse(
            decode(&vpd_as_string)
                .map_err(|e| Errors::parse("Unable to decode vpd", Some(anyhow::Error::from(e))))?
                .as_ref()
        )
        .map_err(|e| {
            Errors::parse("Unable to extract url from string", Some(anyhow::Error::from(e)))
        })?;

        let vpd_json = get_query_param(&url, "presentation_definition")?;

        parse_from_str(&vpd_json)
    }

    async fn get_matching_vcs(&self, vpd: &Vpd) -> Outcome<Vec<String>> {
        info!("Matching Verifiable Credentials for OIDC4VP");
        let mut vcs_id = Vec::new();
        for descriptor in vpd.input_descriptors.clone() {
            let n_vpd = Vpd { id: "temporal_id".to_string(), input_descriptors: vec![descriptor] };
            let vcs = self.match_vc4vp(parse_to_value(&n_vpd)?).await?;
            let vc_id = vcs.first().map(|data| data.id.clone()).ok_or_else(|| {
                Errors::missing_action(
                    MissingAction::Credentials,
                    "There are no VCs that match the specified input descriptor",
                    None
                )
            })?;
            vcs_id.push(vc_id);
        }
        Ok(vcs_id)
    }

    async fn match_vc4vp(&self, vp_def: Value) -> Outcome<Vec<MatchingVCs>> {
        info!("Matching vcs for a specific descriptor");
        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/matchCredentialsForPresentationDefinition",
            self.config.get_wallet_api_url(),
            wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let res = self.client.post(&url, Some(headers), Body::Json(vp_def)).await?;
        match res.status().as_u16() {
            200 => {
                info!("Credentials matched successfully");
                let vc_json: Vec<MatchingVCs> = parse_json_resp(res).await?;
                Ok(vc_json)
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Petition to match credentials failed",
                None
            ))
        }
    }

    async fn present_vp(&self, payload: &OidcUri, vcs_id: Vec<String>) -> Outcome<Option<String>> {
        info!("Presenting Verifiable Presentation");
        let wallet = self.get_wallet().await?;
        let token = self.get_token().await?;
        let did = self.get_did().await?;

        let url = format!(
            "{}/wallet-api/wallet/{}/exchange/usePresentationRequest",
            self.config.get_wallet_api_url(),
            wallet.id
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let body = MatchVCsRequest {
            did,
            presentation_request: payload.uri.clone(),
            selected_credentials: vcs_id
        };

        let res = self.client.post(&url, Some(headers), Body::Json(parse_to_value(&body)?)).await?;
        match res.status().as_u16() {
            200 => {
                info!("Credentials presented successfully");
                // let data: RedirectResponse = res.json().await?;
                match res.json::<Option<RedirectResponse>>().await {
                    Ok(Some(data)) => Ok(Some(data.redirect_uri)),
                    _ => Ok(None)
                }
            }
            status => Err(Errors::wallet(
                &url,
                "POST",
                Some(status),
                "Petition to present credentials failed",
                None
            ))
        }
    }
}
