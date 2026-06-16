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

mod evidence;
mod refresh_service;
mod schema;
mod status;
mod terms_of_use;
mod vc_builder;
mod vc_doc;

pub use evidence::VCEvidence;
pub use refresh_service::VCRefreshService;
pub use schema::*;
pub use status::VCStatus;
pub use terms_of_use::TermsOfUse;
pub use vc_builder::VcDocumentBuilder;
pub use vc_doc::VcDocument;
