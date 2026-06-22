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

use super::HasId;
use crate::errors::Errors;
use crate::types::keys::{Crv, Kty, PrivateKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct KeyPlan {
    pub alias: String,
    pub kty: Kty,
    pub crv: Option<Crv>,
    pub pem: String,
}

#[derive(Serialize, Deserialize)]
pub struct KeyModel {
    pub id: String,
    pub alias: String,
    pub kty: Kty,
    pub crv: Option<Crv>,
    pub pem: String,
}

impl HasId for KeyModel {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Into<KeyModel> for KeyPlan {
    fn into(self) -> KeyModel {
        todo!()
    }
}

impl TryInto<PrivateKey> for KeyModel {
    type Error = Errors;

    fn try_into(self) -> Result<PrivateKey, Self::Error> {
        PrivateKey::from_safe_pem(&self.pem, &self.kty, self.crv.as_ref())
    }
}
