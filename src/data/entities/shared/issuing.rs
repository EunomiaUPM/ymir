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

use sea_orm::ActiveValue;
use sea_orm::entity::prelude::*;
use uuid::Uuid;
use crate::data::entities::IntoOverwriteActive;
use crate::types::vcs::VcTypeConfig;
use crate::utils::create_opaque_token;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "issuing")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    // pub name: String,
    // pub pre_auth_code: String,
    // pub tx_code: String,
    // pub step: bool,
    pub vc_type: VcTypeConfig,
    // pub uri: String,
    // pub token: String,
    // pub aud: String,
    // pub holder_did: Option<String>,
    // pub issuer_did: Option<String>,
    // pub credential_id: String,
    // pub credential: Option<String>,
    pub build_ctx: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct Plan {
    pub id: String,
    // pub name: String,
    pub vc_type: VcTypeConfig,
    // pub aud: String,
    // pub uri: String,
    pub build_ctx: serde_json::Value,
}

impl IntoOverwriteActive<ActiveModel> for Plan {
    fn into_active(self) -> ActiveModel {
        // let code = create_opaque_token();
        // let tx_code = create_opaque_token();
        // let token = create_opaque_token();
        // let credential_id = format!("urn:uuid:{}", Uuid::new_v4().to_string());
        ActiveModel {
            id: ActiveValue::Set(self.id),
            // name: ActiveValue::Set(self.name),
            // pre_auth_code: ActiveValue::Set(code),
            // tx_code: ActiveValue::Set(tx_code),
            // step: ActiveValue::Set(true),
            vc_type: ActiveValue::Set(self.vc_type),
            // uri: ActiveValue::Set(self.uri),
            // token: ActiveValue::Set(token),
            // aud: ActiveValue::Set(self.aud),
            // holder_did: ActiveValue::Set(None),
            // issuer_did: ActiveValue::Set(None),
            // credential_id: ActiveValue::Set(credential_id),
            // credential: ActiveValue::Set(None),
            build_ctx: ActiveValue::Set(self.build_ctx),
        }
    }
}

impl IntoOverwriteActive<ActiveModel> for Model {
    fn into_active(self) -> ActiveModel {
        ActiveModel {
            id: ActiveValue::Set(self.id),
            // name: ActiveValue::Set(self.name),
            // pre_auth_code: ActiveValue::Set(code),
            // tx_code: ActiveValue::Set(tx_code),
            // step: ActiveValue::Set(true),
            vc_type: ActiveValue::Set(self.vc_type),
            // uri: ActiveValue::Set(self.uri),
            // token: ActiveValue::Set(token),
            // aud: ActiveValue::Set(self.aud),
            // holder_did: ActiveValue::Set(None),
            // issuer_did: ActiveValue::Set(None),
            // credential_id: ActiveValue::Set(credential_id),
            // credential: ActiveValue::Set(None),
            build_ctx: ActiveValue::Set(self.build_ctx),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
