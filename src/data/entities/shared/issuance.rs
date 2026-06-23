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

use crate::services::repo::postgres::IntoOverwriteActive;
use crate::types::vcs::{BuildCtx, VcTypeConfig};
use crate::utils::create_opaque_token;
use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "issuance")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub subject_name: String,
    pub pre_auth_code: String,
    pub vc_type_config: Vec<VcTypeConfig>,
    pub token: String,
    pub token_expiration: u32,
    pub nonce: String,
    pub aud: String,
    pub issuer_did: String,
    pub credential_id: String,
    pub credential: Option<String>,
    pub build_ctx: BuildCtx,
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,
    pub subject_name: String,
    pub vc_type_config: Vec<VcTypeConfig>,
    pub aud: String,
    pub issuer_did: String,
    pub build_ctx: BuildCtx,
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        let code = create_opaque_token();
        let token = create_opaque_token();
        let nonce = create_opaque_token();
        let credential_id = format!("urn:uuid:{}", Uuid::new_v4().to_string());
        ActiveModel {
            id: ActiveValue::Set(self.id),
            subject_name: ActiveValue::Set(self.subject_name),
            pre_auth_code: ActiveValue::Set(code),
            vc_type_config: ActiveValue::Set(self.vc_type_config),
            token: ActiveValue::Set(token),
            token_expiration: ActiveValue::Set(600),
            nonce: ActiveValue::Set(nonce),
            aud: ActiveValue::Set(self.aud),
            issuer_did: ActiveValue::Set(self.issuer_did),
            credential_id: ActiveValue::Set(credential_id),
            credential: ActiveValue::Set(None),
            build_ctx: ActiveValue::Set(self.build_ctx),
        }
    }
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            subject_name: ActiveValue::Set(self.subject_name),
            pre_auth_code: ActiveValue::Set(self.pre_auth_code),
            vc_type_config: ActiveValue::Set(self.vc_type_config),
            token: ActiveValue::Set(self.token),
            token_expiration: ActiveValue::Set(self.token_expiration),
            nonce: ActiveValue::Set(self.nonce),
            aud: ActiveValue::Set(self.aud),
            issuer_did: ActiveValue::Set(self.issuer_did),
            credential_id: ActiveValue::Set(self.credential_id),
            credential: ActiveValue::Set(None),
            build_ctx: ActiveValue::Set(self.build_ctx),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
