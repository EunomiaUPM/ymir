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

use crate::modules::WalletModuleTrait;
use crate::errors::AppResult;
use crate::types::dids::{DidBuilder, DidDocument};
use crate::types::secrets::PemHelper;
use crate::types::wallet::WalletInfo;
use crate::types::wallet::waltid::{IsLinked, OidcUri};
use crate::utils::extract_payload;
use crate::data::entities::wallet::vc::Model;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;

#[derive(Deserialize)]
struct RegisterKeyReq {
    #[serde(flatten)]
    pem_helper: PemHelper,
    alias: Option<String>,
}

#[derive(Deserialize)]
struct RegisterDidReq {
    builder: DidBuilder,
    keys_id: Vec<String>,
    alias: Option<String>,
}

pub struct WalletRouter {
    holder: Arc<dyn WalletModuleTrait>,
}

impl WalletRouter {
    pub fn new(holder: Arc<dyn WalletModuleTrait>) -> Self {
        Self { holder }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/is-linked", get(Self::is_linked))
            .route("/link", post(Self::link))
            .route("/key", post(Self::register_key))
            .route("/key/{id}", delete(Self::delete_key))
            .route("/did", get(Self::get_wallet_did).post(Self::register_did))
            .route("/did/{id}", delete(Self::delete_did))
            .route("/credential/{id}", delete(Self::delete_credential))
            .route("/info", get(Self::get_wallet_info))
            .route("/vcs", get(Self::get_wallet_credentials))
            .route("/oidc4vci", post(Self::process_oidc4vci))
            .route("/oidc4vp", post(Self::process_oidc4vp))
            .with_state(self.holder)
    }

    pub fn well_known(&self) -> Router {
        Router::new()
            .route("/.well-known/did.json", get(Self::get_did_doc))
            .with_state(self.holder.clone())
    }

    async fn link(State(holder): State<Arc<dyn WalletModuleTrait>>) -> AppResult<()> {
        holder.link().await
    }

    async fn is_linked(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
    ) -> AppResult<Json<IsLinked>> {
        Ok(Json(holder.is_linked().await))
    }

    async fn register_key(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        payload: Result<Json<RegisterKeyReq>, JsonRejection>,
    ) -> AppResult<StatusCode> {
        let req = extract_payload(payload)?;
        holder.register_key(req.pem_helper, req.alias).await?;
        Ok(StatusCode::CREATED)
    }

    async fn register_did(
        State(holder): State<Arc<dyn WalletModuleTrait>>,
        payload: Result<Json<RegisterDidReq>, JsonRejection>,
    ) -> AppResult<StatusCode> {
        let req = extract_payload(payload)?;
        holder
            .register_did(req.builder, req.keys_id, req.alias)
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
        holder.delete_did(&id).await
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
}
