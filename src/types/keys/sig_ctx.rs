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

use crate::capabilities::Did;
use crate::types::keys::PrivateKey;

pub struct SigningCtx {
    did: Did,
    key: PrivateKey,
    keys_frag: String,
}

impl SigningCtx {
    pub fn new(did: Did, key: PrivateKey, keys_frag: String) -> Self {
        SigningCtx { did, key, keys_frag }
    }
    pub fn did(&self) -> &Did {
        &self.did
    }
    pub fn key(&self) -> &PrivateKey {
        &self.key
    }
    pub fn keys_frag(&self) -> &String {
        &self.keys_frag
    }
}
