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

use crate::data::entities::wallet::vc::Model;
use crate::data::entities::wallet::{did, key};
use crate::errors::AppResult;
use crate::http::routes::wallet;
use crate::modules::WalletModuleTrait;
use crate::types::dids::{DidBuilder, DidDocument, DidService};
use crate::types::wallet::{DidSearch, OidcUri, WalletInfo};
use crate::utils::extract_payload;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;

/// Internal operational payload to register and pair raw asymmetric private keys.
#[derive(Deserialize)]
struct RegisterKeyReq {
    /// Raw private key in PEM format. `kty`/`crv` are derived from it on the wallet side.
    pem: String,
    /// Human-readable custom alias tag to index and reference the cryptographic keypair.
    alias: Option<String>,
}

/// Internal operational payload to execute local Decentralized Identifier (DID) initialization routines.
#[derive(Deserialize)]
struct RegisterDidReq {
    /// Strategy parameters and target DID method constructor layout settings.
    builder: DidBuilder,
    /// Associated list of pre-provisioned relational key IDs to bind into the DID verification methods.
    keys_id: Vec<String>,
    /// Human-readable identity identifier alias.
    alias: Option<String>,
    /// Optional DID Document service entries.
    #[serde(default)]
    service: Option<Vec<DidService>>,
}

/// HTTP API Gateway Router governing the Wallet Module ecosystem.
///
/// Exposes administrative endpoints for key and DID lifecycle tracking, Verifiable Credentials inventories,
/// and standard out-of-band execution entry points for dynamic OID4VCI / OID4VP protocol exchanges.
pub struct WalletRouter {
    holder: Arc<dyn WalletModuleTrait>,
}

impl WalletRouter {
    /// Instantiates a new HTTP network boundary instance wrapping the target functional business module.
    pub fn new(holder: Arc<dyn WalletModuleTrait>) -> Self {
        Self { holder }
    }

    /// Composes and provisions the foundational operational API routing tree bound to its shared module state context.
    ///
    /// # Exposed Map
    /// * `GET  /is-linked`      - Asserts linking execution parameters.
    /// * `POST /link`           - Enforces external ecosystem directory linkages.
    /// * `POST /key`            - Imports raw asymmetric cryptographic key material.
    /// * `DELETE /key/{id}`     - Purges custom key references.
    /// * `GET/POST /did`        - Fetches primary identity string or spawns custom local DIDs.
    /// * `DELETE /did/{id}`     - Drops target DID structural mappings.
    /// * `DELETE /credential/{id}` - Un-links and purges specific credential records.
    /// * `GET  /info`           - Resolves runtime telemetry indicators.
    /// * `GET  /vcs`            - Collects full relational credential arrays.
    /// * `POST /oidc4vci`       - Dispatches inbound OpenID4VCI credential offers.
    /// * `POST /oidc4vp`        - Resolves outbound presentation request validation targets.
    pub fn internal(self) -> Router {
        Router::new()
            .route(wallet::IS_LINKED, get(Self::is_linked))
            .route(wallet::LINK, post(Self::link))
            .route(wallet::KEY, post(Self::register_key))
            .route(wallet::KEYS, get(Self::get_wallet_keys))
            .route(wallet::KEY_BY_ID, delete(Self::delete_key))
            .route(wallet::DID, get(Self::get_wallet_did).post(Self::register_did))
            .route(wallet::DID_BY_ID, delete(Self::delete_did))
            .route(wallet::DID_DEFAULT, post(Self::set_default_did))
            .route(
                wallet::DID_KEY,
                post(Self::add_key_to_did).delete(Self::remove_key_from_did),
            )
            .route(wallet::DID_DEFAULT_KEY, post(Self::set_default_key))
            .route(wallet::CREDENTIAL_BY_ID, delete(Self::delete_credential))
            .route(wallet::INFO, get(Self::get_wallet_info))
            .route(wallet::VCS, get(Self::get_wallet_credentials))
            .route(wallet::OID4VCI, post(Self::process_oidc4vci))
            .route(wallet::OID4VP, post(Self::process_oidc4vp))
            .with_state(self.holder)
    }

    /// Mounts an isolated routing context specifically configured to answer public `did:web` resolution challenges.
    ///
    /// Extracted separately to ensure public resolution hooks can be binded to public network domains (`/.well-known/did.json`)
    /// without coupling corporate administrative boundaries into the open discovery web.
    pub fn well_known(&self) -> Router {
        Router::new()
            .route(wallet::DID_DOC, get(Self::get_did_doc))
            .with_state(self.holder.clone())
    }

    // ===== HTTP HANDLER INNER LOGIC REPRESENTATIONS ==============================================

    async fn link(State(holder): State<Arc<dyn WalletModuleTrait>>) -> AppResult<()> {
        holder.link().await
    }

    async fn is_linked(State(holder): State<Arc<dyn WalletModuleTrait>>) -> AppResult {
        Ok(match holder.is_linked().await {
            true => StatusCode::OK.into_response(),
            false => StatusCode::NOT_FOUND.into_response(),
        })
    }

    async fn register_key(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        payload: Result<Json<RegisterKeyReq>, JsonRejection>,
    ) -> AppResult<StatusCode> {
        let req = extract_payload(payload)?;
        holder.register_key(req.pem, req.alias).await?;
        Ok(StatusCode::CREATED)
    }

    async fn register_did(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        payload: Result<Json<RegisterDidReq>, JsonRejection>,
    ) -> AppResult<StatusCode> {
        let req = extract_payload(payload)?;
        holder
            .register_did(req.builder, req.keys_id, req.alias, req.service)
            .await?;
        Ok(StatusCode::CREATED)
    }

    async fn delete_key(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        Path(id): Path<String>,
    ) -> AppResult<()> {
        holder.delete_key(&id).await
    }

    async fn delete_did(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        Path(id): Path<String>,
    ) -> AppResult<()> {
        holder.delete_did(DidSearch::Id(id)).await
    }

    async fn set_default_did(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        Path(id): Path<String>,
    ) -> AppResult<Json<did::Model>> {
        let model = holder.set_default_did(DidSearch::Id(id)).await?;
        Ok(Json(model))
    }

    async fn add_key_to_did(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        Path((id, key_id)): Path<(String, String)>,
    ) -> AppResult<Json<did::Model>> {
        let model = holder.add_key_to_did(DidSearch::Id(id), key_id).await?;
        Ok(Json(model))
    }

    async fn remove_key_from_did(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        Path((id, key_id)): Path<(String, String)>,
    ) -> AppResult<Json<did::Model>> {
        let model = holder
            .remove_key_from_did(DidSearch::Id(id), key_id)
            .await?;
        Ok(Json(model))
    }

    async fn set_default_key(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        Path((id, key_id)): Path<(String, String)>,
    ) -> AppResult<Json<did::Model>> {
        let model = holder.set_default_key(DidSearch::Id(id), key_id).await?;
        Ok(Json(model))
    }

    async fn delete_credential(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        Path(id): Path<String>,
    ) -> AppResult<()> {
        holder.delete_credential(&id).await
    }

    async fn process_oidc4vci(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        holder.process_oidc4vci(payload).await
    }

    async fn process_oidc4vp(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        holder.process_oidc4vp(payload).await
    }

    async fn get_wallet_did(State(holder): State<Arc<dyn WalletModuleTrait>>) -> AppResult<String> {
        Ok(holder.get_wallet_did().await?)
    }

    async fn get_did_doc(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
    ) -> AppResult<Json<DidDocument>> {
        Ok(Json(holder.get_did_doc().await?))
    }

    async fn get_wallet_info(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
    ) -> AppResult<Json<WalletInfo>> {
        Ok(Json(holder.get_wallet_info().await?))
    }

    async fn get_wallet_credentials(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
    ) -> AppResult<Json<Vec<Model>>> {
        Ok(Json(holder.get_wallet_credentials().await?))
    }

    async fn get_wallet_keys(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
    ) -> AppResult<Json<Vec<key::Model>>> {
        Ok(Json(holder.get_wallet_keys().await?))
    }
}
