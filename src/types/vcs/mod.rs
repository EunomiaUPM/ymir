/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

pub mod claims_v1;
pub mod claims_v2;
mod input_descriptor;
pub mod vc_decision_approval;
pub mod vc_issuer;
pub mod vc_specs;
mod vc_type;
mod vpd;
mod w3c_data_model;
mod gaia_vc;

pub use input_descriptor::InputDescriptor;
pub use vc_type::VcType;
pub use vpd::VPDef;
pub use w3c_data_model::W3cDataModelVersion;
pub use gaia_vc::*;
