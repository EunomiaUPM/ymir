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

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use rand::distributions::Alphanumeric;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use sha2::{Digest, Sha256, Sha384, Sha512};
use crate::data::entities::IntoOverwriteActive;
use crate::types::gnap::grant_request::interact::{FinishMethod, HashMethod, InteractStart};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "recv_interactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String, // RESPONSE
    pub start: Vec<InteractStart>,        // RESPONSE
    pub method: FinishMethod,            // RESPONSE
    pub callback_uri: String,               // RESPONSE
    pub cert: String,              // RESPONSE
    pub client_nonce: String,      // RESPONSE
    pub hash_method: HashMethod,       // RESPONSE
    pub hints: Option<String>,     // RESPONSE
    pub continue_endpoint: String, // RESPONSE
    pub continue_id: String,       // RESPONSE
    pub continue_token: String,    // RESPONSE
    pub continue_wait: Option<i64>,        // RESPONSE
    pub as_nonce: String,          // RANDOM
    pub interact_ref: String,      // RANDOM
    pub hash: String,              // RANDOM
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,                  // REQUEST
    pub start: Vec<InteractStart>,          // REQUEST
    pub method: FinishMethod,              // REQUEST
    pub callback_uri: String,                 // REQUEST
    pub cert: String,                // REQUEST
    pub client_nonce: String,        // REQUEST
    pub hash_method: Option<HashMethod>, // REQUEST
    pub hints: Option<String>,       // REQUEST
    pub grant_endpoint: String,      // REQUEST
    pub continue_endpoint: String,   // RESPONSE
    pub continue_token: String,      // RESPONSE
    pub continue_wait: Option<i64>,        // RESPONSE
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        let as_nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(36)
            .map(char::from)
            .collect();
        let interact_ref: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        let continue_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();

        let hash_method = self.hash_method.unwrap_or(HashMethod::Sha256);

        let hash_input = format!(
            "{}\n{}\n{}\n{}",
            self.client_nonce, as_nonce, interact_ref, self.grant_endpoint
        );

        let hash_result = match &hash_method {
            HashMethod::Sha256 => {
                let mut h = Sha256::new();
                h.update(hash_input.as_bytes());
                h.finalize().to_vec()
            }
            HashMethod::Sha384 => {
                let mut h = Sha384::new();
                h.update(hash_input.as_bytes());
                h.finalize().to_vec()
            }
            HashMethod::Sha512 => {
                let mut h = Sha512::new();
                h.update(hash_input.as_bytes());
                h.finalize().to_vec()
            }
            _ => unreachable!(),
        };


        let cont_endpoint = format!("{}/{}", self.continue_endpoint, continue_id);

        let hash = URL_SAFE_NO_PAD.encode(hash_result);

        ActiveModel {
            id: ActiveValue::Set(self.id),
            start: ActiveValue::Set(self.start),
            method: ActiveValue::Set(self.method),
            callback_uri: ActiveValue::Set(self.callback_uri),
            cert: ActiveValue::Set(self.cert),
            client_nonce: ActiveValue::Set(self.client_nonce),
            hash_method: ActiveValue::Set(hash_method),
            hints: ActiveValue::Set(self.hints),
            continue_endpoint: ActiveValue::Set(cont_endpoint),
            continue_id: ActiveValue::Set(continue_id),
            continue_token: ActiveValue::Set(self.continue_token),
            continue_wait: ActiveValue::Set(self.continue_wait),
            as_nonce: ActiveValue::Set(as_nonce),
            interact_ref: ActiveValue::Set(interact_ref),
            hash: ActiveValue::Set(hash),
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
            cert: ActiveValue::Set(self.cert),
            client_nonce: ActiveValue::Set(self.client_nonce),
            hash_method: ActiveValue::Set(self.hash_method),
            hints: ActiveValue::Set(self.hints),
            continue_endpoint: ActiveValue::Set(self.continue_endpoint),
            continue_id: ActiveValue::Set(self.continue_id),
            continue_token: ActiveValue::Set(self.continue_token),
            continue_wait: ActiveValue::Set(self.continue_wait),
            as_nonce: ActiveValue::Set(self.as_nonce),
            interact_ref: ActiveValue::Set(self.interact_ref),
            hash: ActiveValue::Set(self.hash),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
