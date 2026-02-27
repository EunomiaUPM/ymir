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
use crate::errors::{BadFormat, Errors, MissingAction, Outcome};
use crate::services::client::ClientTrait;
use crate::services::vault::VaultTrait;
use crate::services::vault::global::VaultService;
use crate::types::dids::did_type::DidType;
use crate::types::dids::dids_info::DidsInfo;
use crate::types::http::Body;
use crate::types::jwt::AuthJwtClaims;
use crate::types::secrets::{SemiWalletSecrets, StringHelper};
use crate::types::wallet::{
    CredentialOfferResponse, KeyDefinition, MatchVCsRequest, MatchingVCs, OidcUri,
    RedirectResponse, Vpd, WalletCredentials, WalletInfo, WalletInfoResponse, WalletLoginResponse,
    WalletSession
};
use crate::utils::{
    ParseHeaderExt, ResponseExt, decode_url_safe_no_pad, expect_from_env, get_query_param,
    json_headers, parse_from_slice, parse_from_str, parse_text_resp, parse_to_value
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

    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Body,
        use_auth: bool,
        is_json: bool,
        error_msg: &str
    ) -> Outcome<Response> {
        let url = format!("{}/wallet-api{}", self.config.get_wallet_api_url(), path);
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
            "GET" => self.client.get(&url, Some(headers)).await?,
            "POST" => self.client.post(&url, Some(headers), body).await?,
            "DELETE" => self.client.delete(&url, Some(headers), body).await?,
            _ => return Err(Errors::not_impl(format!("Method {}", method), None))
        };

        if res.status().is_success() {
            Ok(res)
        } else {
            Err(Errors::wallet(&url, method, Some(res.status()), error_msg, None))
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

        let res = self.client.post(&url, Some(json_headers()), Body::Json(body)).await?;

        if res.status().is_success() {
            info!("Wallet account registration successful");
        } else {
            if res.status().as_u16() == 409 {
                warn!("Wallet account has already registered");
            } else {
                return Err(Errors::wallet(
                    &url,
                    "POST",
                    Some(res.status()),
                    "Petition to register Wallet failed",
                    None
                ));
            }
        }

        Ok(())
    }

    async fn login(&self) -> Outcome<()> {
        info!("Login into web wallet");

        let db_path = expect_from_env("VAULT_APP_WALLET");
        let body: SemiWalletSecrets = self.vault.read(None, &db_path).await?;

        let res = self
            .request(
                "POST",
                "/auth/login",
                Body::json(&body)?,
                false,
                true,
                "Petition to login into Wallet failed"
            )
            .await?;

        info!("Wallet login successful");

        let json_res: WalletLoginResponse = res.parse_json().await?;

        let mut wallet_session = self.wallet_session.lock().await;
        wallet_session.account_id = Some(json_res.id);

        let jwt = json_res.token;
        let jwt_parts: Vec<&str> = jwt.split('.').collect();
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
        wallet_session.token = Some(jwt);

        info!("Login data saved successfully");
        Ok(())
    }

    async fn logout(&self) -> Outcome<()> {
        info!("Login out of web wallet");
        self.request(
            "POST",
            "/auth/logout",
            Body::None,
            false,
            true,
            "Petition to logout from Wallet failed"
        )
        .await?;

        info!("Wallet logout successful");
        let mut wallet_session = self.wallet_session.lock().await;
        wallet_session.token = None;
        Ok(())
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

        let did = self.register_did().await?;
        self.set_default_did(did.as_deref()).await?;

        self.retrieve_wallet_info().await?;
        self.retrieve_wallet_dids().await?;

        let mate = self.get_self_mate().await?;
        let minion = self.get_self_minion().await?;

        Ok((mate, minion))
    }

    async fn partial_onboard(&self) -> Outcome<(mates::NewModel, minions::NewModel)> {
        info!("Initializing partial onboarding");

        self.login().await?;
        self.retrieve_wallet_info().await?;
        self.retrieve_wallet_keys().await?;
        self.retrieve_wallet_dids().await?;

        info!("Initialization successful");
        let mate = self.get_self_mate().await?;
        let minion = self.get_self_minion().await?;
        Ok((mate, minion))
    }

    async fn get_self_mate(&self) -> Outcome<mates::NewModel> {
        let did = self.get_did().await?;
        Ok(mates::NewModel {
            participant_id: did.clone(),
            participant_slug: "Myself".to_string(),
            participant_type: "Agent".to_string(),
            base_url: self.config.hosts().get_host(HostType::Http),
            token: None,
            is_me: true
        })
    }

    async fn get_self_minion(&self) -> Outcome<minions::NewModel> {
        let did = self.get_did().await?;
        Ok(minions::NewModel {
            participant_id: did,
            participant_slug: "Myself".to_string(),
            participant_type: "Authority".to_string(),
            base_url: Some(self.config.hosts().get_host(HostType::Http)),
            vc_uri: None,
            is_vc_issued: false,
            is_me: true
        })
    }

    async fn has_onboarded(&self) -> bool {
        match self.get_wallet().await {
            Ok(_) => true,
            Err(_) => false
        }
    }

    // GET FROM STRUCT------------------------------------------------------------------------------------>
    async fn get_wallet(&self) -> Outcome<WalletInfo> {
        info!("Getting wallet data");
        let wallet_session = self.wallet_session.lock().await;

        wallet_session.wallets.first().cloned().ok_or_else(|| {
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
        wallet_session.token.as_ref().cloned().ok_or_else(|| {
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

        let res = self
            .request(
                "GET",
                "/wallet/accounts/wallets",
                Body::None,
                true,
                true,
                "Petition to retrieve Wallet information failed"
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
        info!("Wallet data loaded successfully");
        Ok(())
    }

    async fn retrieve_wallet_keys(&self) -> Outcome<()> {
        info!("Retrieving keys from web wallet");

        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/keys", wallet.id);

        let res = self
            .request(
                "GET",
                &path,
                Body::None,
                true,
                false,
                "Petition to retrieve keys failed"
            )
            .await?;

        info!("Keys retrieved successfully");
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
        info!("Retrieving dids from web wallet");

        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/dids", wallet.id);

        let res = self
            .request(
                "GET",
                &path,
                Body::None,
                true,
                true,
                "Petition to retrieve Wallet DIDs failed"
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

        info!("Wallet Dids data loaded successfully");
        Ok(())
    }

    async fn retrieve_wallet_credentials(&self) -> Outcome<Vec<WalletCredentials>> {
        info!("Retrieving credentials from web wallet");

        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/credentials", wallet.id);

        let res = self
            .request(
                "GET",
                &path,
                Body::None,
                true,
                true,
                "Petition to retrieve Wallet Credentials failed"
            )
            .await?;

        let res: Vec<WalletCredentials> = res.parse_json().await?;
        info!("Wallet Credentials data loaded successfully");
        Ok(res)
    }

    // REGISTER STUFF IN WALLET
    // ----------------------------------------------------------------------------->
    async fn register_key(&self) -> Outcome<()> {
        info!("Registering key in web wallet");

        let wallet = self.get_wallet().await?;
        let priv_key_path = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: StringHelper = self.vault.read(None, &priv_key_path).await?;

        let path = format!("/wallet/{}/keys/import", wallet.id);

        self.request(
            "POST",
            &path,
            Body::Raw(priv_key.data().to_string()),
            true,
            false,
            "Petition to register key failed"
        )
        .await?;

        info!("Key registered successfully");
        Ok(())
    }

    async fn register_did(&self) -> Outcome<Option<String>> {
        info!("Registering did in web wallet");

        let res = match self.config.get_did_type() {
            DidType::Web => self.reg_did_web().await?,
            DidType::Jwk => self.reg_did_jwk().await?,
            DidType::Other => return Err(Errors::not_impl("Other did type", None))
        };
        if res.status().is_success() {
            info!("Did registered successfully");
            let res = res.parse_text().await?;
            debug!("{:#?}", res);
            Ok(Some(res))
        } else {
            if res.status().as_u16() == 409 {
                warn!("Did already exists");
                Ok(None)
            } else {
                Err(Errors::wallet(
                    "http://register_did_in_wallet",
                    "POST",
                    Some(res.status()),
                    "Petition to register key failed",
                    None
                ))
            }
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
            Body::None,
            true,
            true,
            "Petition to register key failed"
        )
        .await
    }

    async fn reg_did_web(&self) -> Outcome<Response> {
        let wallet = self.get_wallet().await?;
        let key_data = self.get_key().await?;

        let options =
            self.config.get_did_web_options().ok_or_else(|| Errors::not_active("did web", None))?;

        let path = match options.path.as_ref() {
            Some(path) => format!("&path={}", path),
            None => "".to_string()
        };

        let path = format!(
            "/wallet/{}/dids/create/web?keyId={}&alias=web&domain={}{}",
            wallet.id, &key_data.key_id.id, options.domain, path
        );

        self.request(
            "POST",
            &path,
            Body::None,
            true,
            true,
            "Petition to register key failed"
        )
        .await
    }

    async fn set_default_did(&self, did: Option<&str>) -> Outcome<()> {
        info!("Setting default did in web wallet");

        let wallet = self.get_wallet().await?;
        let did = match did {
            Some(did) => did.to_string(),
            None => self.get_did().await?
        };

        let path = format!("/wallet/{}/dids/default?did={}", wallet.id, did);

        self.request(
            "POST",
            &path,
            Body::None,
            true,
            true,
            "Petition to set did as default failed"
        )
        .await?;

        info!("Did has been set as default");
        Ok(())
    }

    // DELETE STUFF FROM WALLET
    // --------------------------------------------------------------------------->
    async fn delete_key(&self, key_id: KeyDefinition) -> Outcome<()> {
        info!("Deleting key in web wallet and from internal data");

        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/keys/{}", wallet.id, key_id.key_id.id);

        self.request(
            "DELETE",
            &path,
            Body::None,
            true,
            false,
            "Petition to delete key failed"
        )
        .await?;

        info!("Key deleted successfully from web wallet");
        let mut keys_data = self.key_data.lock().await;
        keys_data.retain(|key| *key != key_id);
        info!("Key deleted successfully from internal data");
        Ok(())
    }

    async fn delete_did(&self, did_info: DidsInfo) -> Outcome<()> {
        info!("Deleting did from web wallet and from internal data");

        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/dids/{}", wallet.id, did_info.did);

        self.request(
            "DELETE",
            &path,
            Body::None,
            true,
            false,
            "Petition to delete key failed"
        )
        .await?;

        info!("Did deleted successfully from web wallet");
        let mut wallet_session = self.first_wallet_mut().await?;
        let wallet = wallet_session.wallets.first_mut().unwrap();

        wallet.dids.retain(|did| *did != did_info);
        info!("Did deleted successfully from internal data");
        Ok(())
    }

    // DO STUFF IN WALLET
    // --------------------------------------------------------------------------->
    async fn resolve_credential_offer(
        &self,
        payload: &OidcUri
    ) -> Outcome<CredentialOfferResponse> {
        info!("Resolving credential offer");

        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/exchange/resolveCredentialOffer", wallet.id);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain;charset=UTF-8"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        let token = self.get_token().await?;
        headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse_header()?);

        let url = format!("{}/wallet-api{}", self.config.get_wallet_api_url(), path);
        let res = self.client.post(&url, Some(headers), Body::Raw(payload.uri.clone())).await?;

        if res.status().is_success() {
            let data: CredentialOfferResponse = res.parse_json().await?;
            info!("Credential offer resolved successfully");
            Ok(data)
        } else {
            Err(Errors::wallet(
                &url,
                "POST",
                Some(res.status()),
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
        let path = format!(
            "/wallet/{}/exchange/resolveIssuerOpenIDMetadata?issuer={}",
            wallet.id, cred_offer.credential_issuer
        );

        let res = self
            .request(
                "GET",
                &path,
                Body::None,
                true,
                true,
                "Petition resolve credential issuer failed"
            )
            .await?;

        let data: Value = res.parse_json().await?;
        info!("Credential issuer resolved successfully");
        Ok(data)
    }

    async fn use_offer_req(
        &self,
        payload: &OidcUri,
        cred_offer: &CredentialOfferResponse
    ) -> Outcome<()> {
        info!("Accepting credential");

        let wallet = self.get_wallet().await?;
        let did = self.get_did().await?;

        let path = format!(
            "/wallet/{}/exchange/useOfferRequest?did={}&requireUserInput=false&pinOrTxCode={}",
            wallet.id, did, cred_offer.grants.pre_authorized_code.pre_authorized_code
        );

        let res = self
            .request(
                "POST",
                &path,
                Body::Raw(payload.uri.clone()),
                true,
                true,
                "Petition accept credential issuer failed"
            )
            .await?;

        let data: Value = res.parse_json().await?;
        info!("Credential accepted successfully");
        debug!("{:#?}", data);
        Ok(())
    }

    async fn get_vpd(&self, payload: &OidcUri) -> Outcome<Vpd> {
        info!("Joining exchange");

        let wallet = self.get_wallet().await?;
        let path = format!("/wallet/{}/exchange/resolvePresentationRequest", wallet.id);

        let res = self
            .request(
                "POST",
                &path,
                Body::Raw(payload.uri.clone()),
                true,
                false,
                "Error joining the exchange"
            )
            .await?;

        info!("Joined the exchange successful");
        let vpd = parse_text_resp(res).await?;
        let vpd = self.parse_vpd(&vpd)?;
        Ok(vpd)
    }

    fn parse_vpd(&self, vpd_as_string: &str) -> Outcome<Vpd> {
        info!("Parsing Vpd");

        let url = Url::parse(
            decode(&vpd_as_string)
                .map_err(|e| Errors::parse("Unable to decode vpd", Some(Box::new(e))))?
                .as_ref()
        )
        .map_err(|e| Errors::parse("Unable to extract url from string", Some(Box::new(e))))?;

        let vpd_json = get_query_param(&url, "presentation_definition")?;

        parse_from_str(&vpd_json)
    }

    async fn get_matching_vcs(&self, vpd: &Vpd) -> Outcome<Vec<String>> {
        info!("Matching Verifiable Credentials for OIDC4VP");
        let mut vcs_id = Vec::with_capacity(vpd.input_descriptors.len());
        for descriptor in &vpd.input_descriptors {
            let n_vpd =
                Vpd { id: "temporal_id".to_string(), input_descriptors: vec![descriptor.clone()] };
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

        let path = format!(
            "/wallet/{}/exchange/matchCredentialsForPresentationDefinition",
            wallet.id
        );

        let res = self
            .request(
                "POST",
                &path,
                Body::Json(vp_def),
                true,
                true,
                "Petition to match credentials failed"
            )
            .await?;

        info!("Credentials matched successfully");
        let vc_json: Vec<MatchingVCs> = res.parse_json().await?;
        Ok(vc_json)
    }

    async fn present_vp(&self, payload: &OidcUri, vcs_id: Vec<String>) -> Outcome<Option<String>> {
        info!("Presenting Verifiable Presentation");
        let wallet = self.get_wallet().await?;
        let did = self.get_did().await?;

        let path = format!("/wallet/{}/exchange/usePresentationRequest", wallet.id);

        let body = MatchVCsRequest {
            did,
            presentation_request: payload.uri.clone(),
            selected_credentials: vcs_id
        };

        let res = self
            .request(
                "POST",
                &path,
                Body::json(&body)?,
                true,
                true,
                "Petition to present credentials failed"
            )
            .await?;

        info!("Credentials presented successfully");
        // let data: RedirectResponse = res.json().await?;
        match res.json::<Option<RedirectResponse>>().await {
            Ok(Some(data)) => Ok(Some(data.redirect_uri)),
            _ => Ok(None)
        }
    }
}
