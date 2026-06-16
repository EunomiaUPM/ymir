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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use rand::Rng;
use rand::distributions::Alphanumeric;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;

use crate::data::entities::IntoOverwriteActive;
use crate::types::gnap::grant_request::interact::{FinishMethod, HashMethod, InteractStart};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sent_interactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,                        // REQUEST
    pub start: Vec<InteractStart>,         // REQUEST
    pub method: FinishMethod,              // REQUEST
    pub callback_uri: String,                       // REQUEST
    pub client_nonce: String,              // RANDOM
    pub hash_method: HashMethod,           // REQUEST
    pub hints: Option<String>,             // REQUEST
    pub continue_endpoint: Option<String>, // RESPONSE
    pub continue_token: Option<String>,    // RESPONSE
    pub continue_wait: Option<i64>,        // RESPONSE
    pub as_nonce: Option<String>,          // RESPONSE
    pub oidc_vp_uri: Option<String>,       // RESPONSE
    pub interact_ref: Option<String>,      // POST-RESPONSE
    pub hash: Option<String>,              // POST-RESPONSE
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,                  // REQUEST
    pub start: Vec<InteractStart>,          // REQUEST
    pub method: FinishMethod,              // REQUEST
    pub callback_uri: String,                 // REQUEST
    pub hash_method: Option<HashMethod>, // REQUEST
    pub hints: Option<String>,       // REQUEST
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(36)
            .map(char::from)
            .collect();
        let hash_method = self.hash_method.unwrap_or(HashMethod::Sha256);
        ActiveModel {
            id: ActiveValue::Set(self.id),
            start: ActiveValue::Set(self.start),
            method: ActiveValue::Set(self.method),
            callback_uri: ActiveValue::Set(self.callback_uri),
            client_nonce: ActiveValue::Set(nonce),
            hash_method: ActiveValue::Set(hash_method),
            hints: ActiveValue::Set(self.hints),
            continue_endpoint: ActiveValue::Set(None),
            continue_token: ActiveValue::Set(None),
            continue_wait: ActiveValue::Set(None),
            as_nonce: ActiveValue::Set(None),
            oidc_vp_uri: ActiveValue::Set(None),
            interact_ref: ActiveValue::Set(None),
            hash: ActiveValue::Set(None),
        }
    }
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            start: ActiveValue::Set(self.start),
            method: ActiveValue::Set(self.method),
            callback_uri: ActiveValue::Set(self.callback_uri),
            client_nonce: ActiveValue::Set(self.client_nonce),
            hash_method: ActiveValue::Set(self.hash_method),
            hints: ActiveValue::Set(self.hints),
            continue_endpoint: ActiveValue::Set(self.continue_endpoint),
            continue_token: ActiveValue::Set(self.continue_token),
            continue_wait: ActiveValue::Set(self.continue_wait),
            as_nonce: ActiveValue::Set(self.as_nonce),
            oidc_vp_uri: ActiveValue::Set(self.oidc_vp_uri),
            interact_ref: ActiveValue::Set(self.interact_ref),
            hash: ActiveValue::Set(self.hash),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
