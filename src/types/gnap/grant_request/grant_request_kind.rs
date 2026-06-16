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
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

use super::access::AccessRequest;
use sea_orm::{DeriveActiveEnum, EnumIter};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GrantRequestKind {
    AccessToken { access_token: AccessRequest },
    CredentialRequest { credential_request: AccessRequest },
}

#[derive(Clone, Debug, Eq, PartialEq, DeriveActiveEnum, EnumIter, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "grant_kind")]
pub enum GrantKind {
    #[sea_orm(string_value = "AccessToken")]
    AccessToken,
    #[sea_orm(string_value = "CredentialRequest")]
    CredentialRequest,
}

impl Display for GrantKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GrantKind::AccessToken => write!(f, "AccessToken"),
            GrantKind::CredentialRequest => write!(f, "CredentialRequest"),
        }
    }
}