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

use async_trait::async_trait;
use reqwest::{Response, Url};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{Mutex, MutexGuard};
use tracing::{debug, info};
use urlencoding::decode;

use super::config::FafnirConfig;
use crate::config::traits::{DidConfigTrait, HostsConfigTrait, WalletConfigTrait};
use crate::config::types::HostType;
use crate::data::entities::{mates, minions};
use crate::errors::{BadFormat, Errors, MissingAction, Outcome};
use crate::services::client::ClientTrait;
use crate::services::vault::{VaultService, VaultTrait};
use crate::services::wallet::WalletTrait;
use crate::types::dids::{DidBuilder, DidDocument, DidType, WebDid};
use crate::types::http::Body;
use crate::types::issuing::VCCredOffer;
use crate::types::keys::{Crv, KeyData, Kty};
use crate::types::secrets::StringHelper;
use crate::types::vcs::VPDef;
use crate::types::wallet::fafnir::{
    DidEntry, DidEntryReq, KeyEntry, KeyEntryReq, VcBodyType, VcEntry,
};
use crate::types::wallet::waltid::{
    CredentialOfferResponse, DidsInfo, KeyDefinition, KeyInfo, MatchingVCs, WalletCredentials,
    WalletInfo, WalletSession,
};
use crate::utils::{expect_from_env, get_query_param, json_headers, ResponseExt};

/// Cliente que implementa `WalletTrait` hablando por HTTP con una
/// `fafnir-wallet` remota. Equivalente funcional a `WaltIdService`,
/// pero contra los endpoints que expone fafnir.
///
/// Asume:
///   - La fafnir-wallet ya tiene una DID por defecto creada (key +
///     DID via `POST /keys/new` y `POST /dids/new`). Este service NO
///     auto-genera identidades.
///   - No hay autenticación/sesión: la wallet es local al despliegue.
pub struct FafnirService {
    client: Arc<dyn ClientTrait>,
    config: FafnirConfig,
    vault: Arc<VaultService>,
    /// El trait `WalletTrait::first_wallet_mut` exige devolver un
    /// `MutexGuard<WalletSession>`. Como en fafnir local no hay sesión
    /// real, mantenemos uno sintético para cumplir la firma.
    wallet_session: Arc<Mutex<WalletSession>>,
}

impl FafnirService {
    pub fn new(
        config: FafnirConfig,
        client: Arc<dyn ClientTrait>,
        vault: Arc<VaultService>,
    ) -> Self {
        Self {
            config,
            client,
            vault,
            wallet_session: Arc::new(Mutex::new(WalletSession {
                account_id: Some("fafnir-local".to_string()),
                token: None,
                token_exp: None,
                wallets: vec![],
            })),
        }
    }

    fn wallet_base(&self) -> String {
        self.config.get_wallet_api_url()
    }
}

#[derive(Serialize)]
struct UriPayload<'a> {
    uri: &'a str,
}

#[async_trait]
impl WalletTrait for FafnirService {
    // ════════════════════════ BASIC ═══════════════════════════════════
    //
    // fafnir-wallet no tiene sesión. Onboarding solo verifica que ya
    // existe un DID por defecto creado.

    async fn onboard(&self) -> Outcome<(mates::NewModel, minions::NewModel)> {
        info!("FafnirService: onboard");

        // Si ya estamos onboardeados (default DID existe), devolvemos
        // mate/minion del DID existente y no creamos nada nuevo.
        // Hace `onboard` idempotente.
        if self.has_onboarded().await {
            debug!("FafnirService: already onboarded");
            return Ok((self.get_self_mate().await?, self.get_self_minion().await?));
        }

        // Bootstrap: leer PEM del vault → crear key → crear did:jwk →
        // marcarlo como default. Imita lo que walt-id hace en sus
        // endpoints `keys/import` + `dids/create/jwk` + `dids/default`.
        info!("FafnirService: bootstrapping default key + DID");

        let priv_key_path = expect_from_env("VAULT_APP_PRIV_KEY");
        let priv_key: StringHelper = self.vault.read(None, &priv_key_path).await?;
        let pem = priv_key.data();
        let (kty, crv) = detect_kty_crv(pem)?;

        // 1) POST /keys/new
        let key_url = format!("{}/keys/new", self.wallet_base());
        let key_req = KeyEntryReq {
            alias: "default".to_string(),
            kty,
            crv,
            pem: pem.to_string(),
        };
        let key_entry: KeyEntry = self
            .client
            .post(&key_url, Some(json_headers()), Body::json(&key_req)?)
            .await?
            .parse_json()
            .await?;
        debug!("FafnirService: key creada (id={})", key_entry.id);

        // 2) POST /dids/new — el tipo de DID viene del config
        //    (`did_config.r#type`), mimando lo que hace `WaltIdService`
        //    en su `register_did`.
        let did_url = format!("{}/dids/new", self.wallet_base());
        let did_builder = match self.config.get_did_type() {
            DidType::Jwk => DidBuilder::Jwk,
            DidType::Web => DidBuilder::Web(build_web_did(&self.config)?),
        };
        let did_req = DidEntryReq {
            alias: "default".to_string(),
            r#type: did_builder,
            keys: vec![key_entry.id.clone()],
            service: None,
        };
        let did_entry: DidEntry = self
            .client
            .post(&did_url, Some(json_headers()), Body::json(&did_req)?)
            .await?
            .parse_json()
            .await?;
        debug!(
            "FafnirService: did creado (id={}, did={})",
            did_entry.id, did_entry.did
        );

        // 3) POST /dids/{id}/default
        let set_default_url = format!("{}/dids/{}/default", self.wallet_base(), did_entry.id);
        self.client
            .post(&set_default_url, Some(json_headers()), Body::None)
            .await?;
        info!(
            "FafnirService: onboarding completo, default did = {}",
            did_entry.did
        );

        // Construimos mate/minion directamente sin volver a llamar a la
        // wallet — ya tenemos el DID.
        let host = self.config.get_host(HostType::Http);
        let mate = mates::NewModel {
            participant_id: did_entry.did.clone(),
            participant_slug: "Myself".to_string(),
            participant_type: "Agent".to_string(),
            base_url: host.clone(),
            token: None,
            extra_fields: None,
            is_me: true,
        };
        let minion = minions::NewModel {
            participant_id: did_entry.did,
            participant_slug: "Myself".to_string(),
            participant_type: "Authority".to_string(),
            base_url: Some(host),
            vc_uri: None,
            is_vc_issued: false,
            is_me: true,
        };
        Ok((mate, minion))
    }

    async fn partial_onboard(&self) -> Outcome<(mates::NewModel, minions::NewModel)> {
        info!("FafnirService: partial_onboard");
        // Lo llama `CoreWalletTrait::onboard` solo cuando
        // `has_onboarded` es `true`. No creamos nada — solo
        // recuperamos los modelos del DID existente.
        if !self.has_onboarded().await {
            return Err(Errors::missing_action(
                MissingAction::Did,
                "partial_onboard sin DID por defecto — llama a onboard primero",
                None,
            ));
        }
        Ok((self.get_self_mate().await?, self.get_self_minion().await?))
    }

    async fn get_self_mate(&self) -> Outcome<mates::NewModel> {
        let did = self.get_did().await?;
        Ok(mates::NewModel {
            participant_id: did,
            participant_slug: "Myself".to_string(),
            participant_type: "Agent".to_string(),
            base_url: self.config.get_host(HostType::Http),
            token: None,
            extra_fields: None,
            is_me: true,
        })
    }

    async fn get_self_minion(&self) -> Outcome<minions::NewModel> {
        let did = self.get_did().await?;
        Ok(minions::NewModel {
            participant_id: did,
            participant_slug: "Myself".to_string(),
            participant_type: "Authority".to_string(),
            base_url: Some(self.config.get_host(HostType::Http)),
            vc_uri: None,
            is_vc_issued: false,
            is_me: true,
        })
    }

    async fn has_onboarded(&self) -> bool {
        self.fetch_default_did().await.is_ok()
    }

    // ════════════════════════ GET FROM MANAGER ════════════════════════
    //
    // En walt-id estos getters leen estado cacheado. Aquí hacemos fetch
    // on-demand contra la fafnir-wallet, sin cache.

    async fn get_wallet(&self) -> Outcome<WalletInfo> {
        let dids = self.fetch_all_dids().await?;
        Ok(WalletInfo {
            id: "fafnir-local".to_string(),
            name: "fafnir-wallet".to_string(),
            created_on: String::new(),
            added_on: String::new(),
            permission: String::new(),
            dids: dids.into_iter().map(did_entry_to_info).collect(),
        })
    }

    async fn first_wallet_mut(&self) -> Outcome<MutexGuard<'_, WalletSession>> {
        Ok(self.wallet_session.lock().await)
    }

    async fn get_did(&self) -> Outcome<String> {
        Ok(self.fetch_default_did().await?.did)
    }

    async fn get_token(&self) -> Outcome<String> {
        Ok(String::new())
    }

    async fn get_did_doc(&self) -> Outcome<DidDocument> {
        Ok(self.fetch_default_did().await?.did_document)
    }

    async fn get_key(&self) -> Outcome<KeyDefinition> {
        let did = self.fetch_default_did().await?;
        let key_id = did.keys.first().ok_or_else(|| {
            Errors::missing_action(MissingAction::Key, "default DID has no keys", None)
        })?;
        let key_entry = self.fetch_key(key_id).await?;
        Ok(key_entry_to_definition(&key_entry))
    }

    // ════════════════════════ RETRIEVE FROM WALLET ════════════════════
    //
    // No mantenemos caché: las RETRIEVE_* son no-ops y las
    // retrieve_wallet_credentials hace fetch directo.

    async fn retrieve_wallet_info(&self) -> Outcome<()> {
        Ok(())
    }
    async fn retrieve_wallet_keys(&self) -> Outcome<()> {
        Ok(())
    }
    async fn retrieve_wallet_dids(&self) -> Outcome<()> {
        Ok(())
    }

    async fn retrieve_wallet_credentials(&self) -> Outcome<Vec<WalletCredentials>> {
        let vcs = self.fetch_all_vcs().await?;
        Ok(vcs.into_iter().map(vc_entry_to_walt).collect())
    }

    // ════════════════════════ REGISTER STUFF ══════════════════════════
    //
    // En fafnir-wallet las keys/DIDs se crean vía POST directos a la
    // API. El onboarding automático de walt-id no aplica.

    async fn register_key(&self) -> Outcome<()> {
        Ok(())
    }

    async fn register_did(&self) -> Outcome<Option<String>> {
        Ok(None)
    }

    async fn reg_did_jwk(&self) -> Outcome<Response> {
        Err(Errors::not_impl(
            "FafnirService: usa POST /dids/new directamente en la wallet",
            None,
        ))
    }

    async fn reg_did_web(&self) -> Outcome<Response> {
        Err(Errors::not_impl(
            "FafnirService: usa POST /dids/new directamente en la wallet",
            None,
        ))
    }

    async fn set_default_did(&self, did: Option<&str>) -> Outcome<()> {
        info!("FafnirService: set_default_did");
        let target_did = match did {
            Some(d) => d.to_string(),
            None => self.get_did().await?,
        };
        // La wallet identifica los DIDs por `id` interno (UUID), no por
        // el string did:jwk:.... Tenemos que resolverlo.
        let all = self.fetch_all_dids().await?;
        let target = all.iter().find(|d| d.did == target_did).ok_or_else(|| {
            Errors::missing_resource(&target_did, "DID not stored in wallet", None)
        })?;
        let url = format!("{}/dids/{}/default", self.wallet_base(), target.id);
        self.client
            .post(&url, Some(json_headers()), Body::None)
            .await?;
        Ok(())
    }

    // ════════════════════════ DELETE ══════════════════════════════════

    async fn delete_key(&self, _key: KeyDefinition) -> Outcome<()> {
        // En walt-id esto se usa en `onboard` para limpiar la wallet
        // recién creada. En fafnir local no hay limpieza automática.
        Ok(())
    }

    async fn delete_did(&self, _did_info: DidsInfo) -> Outcome<()> {
        // Idem.
        Ok(())
    }

    async fn delete_vc(&self, id: &str) -> Outcome<()> {
        info!("FafnirService: delete_vc({})", id);
        let url = format!("{}/vcs/{}", self.wallet_base(), id);
        self.client.delete(&url, None, Body::None).await?;
        Ok(())
    }

    // ════════════════════════ OIDC atómicos walt-id ════════════════════
    //
    // Estos no los expone fafnir-wallet con tipos walt-id; el flujo
    // OIDC va por `process_oidc4vci` / `process_oidc4vp`.

    async fn resolve_credential_offer(&self, _uri: &str) -> Outcome<CredentialOfferResponse> {
        Err(Errors::not_impl(
            "FafnirService: usa process_oidc4vci en lugar de resolve_credential_offer",
            None,
        ))
    }

    async fn resolve_credential_issuer(
        &self,
        _cred_offer: &CredentialOfferResponse,
    ) -> Outcome<Value> {
        Err(Errors::not_impl(
            "FafnirService: usa process_oidc4vci en lugar de resolve_credential_issuer",
            None,
        ))
    }

    async fn use_offer_req(
        &self,
        _uri: &str,
        _cred_offer: &CredentialOfferResponse,
    ) -> Outcome<()> {
        Err(Errors::not_impl(
            "FafnirService: usa process_oidc4vci en lugar de use_offer_req",
            None,
        ))
    }

    async fn get_vpd(&self, uri: &str) -> Outcome<VPDef> {
        info!("FafnirService: get_vpd");
        let url = format!("{}/vp/resolve_request", self.wallet_base());
        self.client
            .post(&url, Some(json_headers()), Body::json(&UriPayload { uri })?)
            .await?
            .parse_json()
            .await
    }

    fn parse_vpd(&self, vpd_as_string: &str) -> Outcome<VPDef> {
        let url = Url::parse(
            decode(vpd_as_string)
                .map_err(|e| Errors::parse("Unable to decode vpd", Some(Box::new(e))))?
                .as_ref(),
        )
        .map_err(|e| Errors::parse("Unable to extract url from string", Some(Box::new(e))))?;
        let vpd_json = get_query_param(&url, "presentation_definition")?;
        Ok(serde_json::from_str(&vpd_json)?)
    }

    async fn get_matching_vcs(&self, _vpd: &VPDef) -> Outcome<Vec<String>> {
        Err(Errors::not_impl(
            "FafnirService: el matching se hace dentro de process_oidc4vp",
            None,
        ))
    }

    async fn match_vc4vp(&self, _vp_def: Value) -> Outcome<Vec<MatchingVCs>> {
        Err(Errors::not_impl(
            "FafnirService: el matching se hace dentro de process_oidc4vp",
            None,
        ))
    }

    async fn present_vp(&self, _uri: &str, _vcs_id: Vec<String>) -> Outcome<Option<String>> {
        Err(Errors::not_impl(
            "FafnirService: usa process_oidc4vp en lugar de present_vp",
            None,
        ))
    }

    // ════════════════════════ OIDC alto nivel ═════════════════════════
    //
    // Un único round-trip: la wallet hace todo el flujo internamente.
    // Los endpoints atómicos `/vci/*` y `/vp/*` siguen disponibles si
    // en algún momento queremos pasos intermedios (mostrar offer al
    // usuario, etc.); para el flujo normal no nos compensa el extra
    // round-trip.

    async fn process_oidc4vci(&self, uri: &str) -> Outcome<()> {
        info!("FafnirService: process_oidc4vci({})", uri);
        let url = format!("{}/oid4vci", self.wallet_base());
        self.client
            .post(
                &url,
                Some(json_headers()),
                Body::json(&UriPayload { uri })?,
            )
            .await?;
        info!("FafnirService: VC aceptada y almacenada");
        Ok(())
    }

    async fn process_oidc4vp(&self, uri: &str) -> Outcome<Option<String>> {
        info!("FafnirService: process_oidc4vp({})", uri);
        let url = format!("{}/oid4vp", self.wallet_base());
        let resp = self
            .client
            .post(
                &url,
                Some(json_headers()),
                Body::json(&UriPayload { uri })?,
            )
            .await?;

        // La wallet responde 200 vacío o 200 con JSON string (redirect).
        let text = resp.parse_text().await.unwrap_or_default();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        // El handler devuelve `Json(url)`, así que llega como `"..."`.
        match serde_json::from_str::<String>(trimmed) {
            Ok(s) => Ok(Some(s)),
            Err(_) => Ok(Some(text)),
        }
    }
}

// ───────────────────────────── helpers internos ──────────────────────────

impl FafnirService {
    async fn fetch_default_did(&self) -> Outcome<DidEntry> {
        let url = format!("{}/dids/default", self.wallet_base());
        self.client
            .get(&url, Some(json_headers()))
            .await?
            .parse_json()
            .await
    }

    async fn fetch_all_dids(&self) -> Outcome<Vec<DidEntry>> {
        let url = format!("{}/dids/all", self.wallet_base());
        self.client
            .get(&url, Some(json_headers()))
            .await?
            .parse_json()
            .await
    }

    async fn fetch_key(&self, id: &str) -> Outcome<KeyEntry> {
        let url = format!("{}/keys/{}", self.wallet_base(), id);
        self.client
            .get(&url, Some(json_headers()))
            .await?
            .parse_json()
            .await
    }

    async fn fetch_all_vcs(&self) -> Outcome<Vec<VcEntry>> {
        let url = format!("{}/vcs/all", self.wallet_base());
        self.client
            .get(&url, Some(json_headers()))
            .await?
            .parse_json()
            .await
    }
}

// ───────────────────────────── mappers fafnir → walt-id ──────────────────

fn did_entry_to_info(did: DidEntry) -> DidsInfo {
    DidsInfo {
        document: serde_json::to_string(&did.did_document).unwrap_or_default(),
        did: did.did,
        alias: did.alias,
        key_id: did.keys.into_iter().next().unwrap_or_default(),
        default: did.default,
        created_on: String::new(),
    }
}

fn key_entry_to_definition(key: &KeyEntry) -> KeyDefinition {
    let algorithm = match (&key.kty, key.crv.as_ref()) {
        (Kty::Okp, Some(Crv::Ed25519)) => "Ed25519".to_string(),
        (Kty::Rsa, _) => "RSA".to_string(),
        _ => format!("{}", key.kty),
    };
    KeyDefinition {
        algorithm,
        crypto_provider: "fafnir".to_string(),
        key_id: KeyInfo {
            id: key.id.clone(),
        },
        key_pair: Value::Null,
        keyset_handle: None,
    }
}

/// Construye un `WebDid` a partir de la `DidConfig` cuando el operador
/// pide `did:web` como tipo de DID. El id se compone con el formato
/// estándar `did:web:<domain>[:<path-con-:-separando-segmentos>]`.
fn build_web_did(config: &FafnirConfig) -> Outcome<WebDid> {
    let domain = config.get_did_web_domain().ok_or_else(|| {
        Errors::format(
            BadFormat::Sent,
            "did:web requested but config.did.did_web_options.domain is missing",
            None,
        )
    })?;
    let path = config.get_did_web_path().map(|p| p.to_string());

    let did_id = match path.as_deref().filter(|p| !p.is_empty()) {
        Some(p) => format!("did:web:{}:{}", domain, p),
        None => format!("did:web:{}", domain),
    };

    Ok(WebDid::new(
        did_id.clone(),
        did_id,
        domain.to_string(),
        path,
        None,
        None,
    ))
}

/// Decide `(kty, crv)` para mandarlos al endpoint `POST /keys/new` de
/// fafnir-wallet, probando los parsers PKCS#8 conocidos sobre el PEM
/// hasta acertar. Imita la idea de `Key::try_weird_from` pero sin
/// construir la `Key` entera.
fn detect_kty_crv(pem: &str) -> Outcome<(Kty, Option<Crv>)> {
    if KeyData::build_ed25519(pem).is_ok() {
        return Ok((Kty::Okp, Some(Crv::Ed25519)));
    }
    if KeyData::build_rsa(pem).is_ok() {
        return Ok((Kty::Rsa, None));
    }
    Err(Errors::format(
        BadFormat::Received,
        "VAULT_APP_PRIV_KEY is not a valid Ed25519/RSA PKCS#8 PEM",
        None,
    ))
}

fn vc_entry_to_walt(vc: VcEntry) -> WalletCredentials {
    let (format, document) = match &vc.r#type {
        VcBodyType::Jwt(s) => ("jwt_vc_json".to_string(), s.clone()),
        VcBodyType::Value(v) => ("ldp_vc".to_string(), v.to_string()),
    };
    WalletCredentials {
        id: vc.id,
        format,
        pending: false,
        wallet: "fafnir-local".to_string(),
        added_on: String::new(),
        disclosures: String::new(),
        document,
        parsed_document: vc.parsed_document,
    }
}
