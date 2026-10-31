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

use serde::{Deserialize, Serialize};

mod did_search;
mod identity;
mod key_ref;
mod oidc_uri;
mod wallet_info;
pub mod waltid;

pub use did_search::DidSearch;
pub use identity::Identity;
pub use key_ref::KeyRef;
pub use oidc_uri::OidcUri;
pub use wallet_info::WalletInfo;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum WalletInstance {
    #[default]
    Fafnir,
    WaltId,
}
