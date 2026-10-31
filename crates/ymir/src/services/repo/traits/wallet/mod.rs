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

mod did_trait;
mod key_trait;
mod vc_trait;

pub use did_trait::DidRepoTrait;
#[cfg(feature = "mock")]
pub use did_trait::MockDidRepoTrait;
pub use key_trait::KeyRepoTrait;
#[cfg(feature = "mock")]
pub use key_trait::MockKeyRepoTrait;
#[cfg(feature = "mock")]
pub use vc_trait::MockVcRepoTrait;
pub use vc_trait::VcRepoTrait;
