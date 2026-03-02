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

use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde_json::Value;

use crate::core_traits::CoreWalletTrait;
use crate::errors::AppResult;
use crate::types::dids::dids_info::DidsInfo;
use crate::types::wallet::{KeyDefinition, OidcUri, WalletCredentials, WalletInfo};
use crate::utils::extract_payload;

pub struct WalletRouter {
    holder: Arc<dyn CoreWalletTrait>
}

impl WalletRouter {
    pub fn new(holder: Arc<dyn CoreWalletTrait>) -> Self { Self { holder } }

    pub fn router(self) -> Router {
        Router::new()
            .route("/register", post(Self::register))
            .route("/login", post(Self::login))
            .route("/logout", post(Self::logout))
            .route("/onboard", post(Self::onboard))
            .route("/partial-onboard", post(Self::partial_onboard))
            .route("/link", post(Self::link))
            .route("/key", post(Self::register_key))
            .route("/did", post(Self::register_did))
            .route("/key", delete(Self::delete_key))
            .route("/did.json", get(Self::did_json))
            .route("/did", delete(Self::delete_did))
            .route("/did", get(Self::get_wallet_did))
            .route("/info", get(Self::get_wallet_info))
            .route("/vcs", get(Self::get_wallet_credentials))
            .route("/oidc4vci", post(Self::process_oidc4vci))
            .route("/oidc4vp", post(Self::process_oidc4vp))
            .with_state(self.holder)
    }

    pub fn well_known(&self) -> Router {
        Router::new()
            .route("/.well-known/did.json", get(Self::did_json))
            .with_state(self.holder.clone())
    }

    async fn register(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<StatusCode> {
        holder.register().await?;
        Ok(StatusCode::CREATED)
    }

    async fn login(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<()> {
        holder.login().await
    }

    async fn logout(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<()> {
        holder.logout().await
    }

    async fn onboard(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<StatusCode> {
        holder.onboard().await?;
        Ok(StatusCode::CREATED)
    }

    async fn partial_onboard(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<()> {
        holder.partial_onboard().await
    }

    async fn link(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<()> {
        holder.link().await
    }

    async fn register_key(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<StatusCode> {
        holder.register_key().await?;
        Ok(StatusCode::CREATED)
    }

    async fn register_did(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<StatusCode> {
        holder.register_did().await?;
        Ok(StatusCode::CREATED)
    }

    async fn delete_key(
        State(holder): State<Arc<dyn CoreWalletTrait>>,
        payload: Result<Json<KeyDefinition>, JsonRejection>
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        holder.delete_key(payload).await
    }

    async fn delete_did(
        State(holder): State<Arc<dyn CoreWalletTrait>>,
        payload: Result<Json<DidsInfo>, JsonRejection>
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        holder.delete_did(payload).await
    }

    async fn did_json(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<Json<Value>> {
        let doc = holder.get_did_doc().await?;
        Ok(Json(doc))
    }

    async fn process_oidc4vci(
        State(holder): State<Arc<dyn CoreWalletTrait>>,
        payload: Result<Json<OidcUri>, JsonRejection>
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        holder.process_oidc4vci(payload).await
    }

    async fn process_oidc4vp(
        State(holder): State<Arc<dyn CoreWalletTrait>>,
        payload: Result<Json<OidcUri>, JsonRejection>
    ) -> AppResult {
        let payload = extract_payload(payload)?;
        let res = holder.process_oidc4vp(payload).await?;
        Ok(match res {
            Some(data) => Redirect::to(&data).into_response(),
            None => StatusCode::OK.into_response()
        })
    }

    async fn get_wallet_did(State(holder): State<Arc<dyn CoreWalletTrait>>) -> AppResult<String> {
        Ok(holder.get_wallet_did().await?)
    }

    async fn get_wallet_info(
        State(holder): State<Arc<dyn CoreWalletTrait>>
    ) -> AppResult<Json<WalletInfo>> {
        Ok(Json(holder.get_wallet_info().await?))
    }

    async fn get_wallet_credentials(
        State(holder): State<Arc<dyn CoreWalletTrait>>
    ) -> AppResult<Json<Vec<WalletCredentials>>> {
        Ok(Json(holder.get_wallet_credentials().await?))
    }
}
