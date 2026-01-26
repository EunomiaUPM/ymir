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

mod basic_repo_trait;
mod business_mates_trait;
mod issuing_trait;
mod mates_trait;
mod minions_trait;
mod recv_interaction_trait;
mod recv_request_trait;
mod recv_verification_trait;
mod req_interaction_trait;
mod req_request_trait;
mod req_vc_trait;
mod req_verification_trait;
mod token_requirements_trait;
mod vc_request_trait;

pub use basic_repo_trait::BasicRepoTrait;
pub use business_mates_trait::BusinessMatesRepoTrait;
pub use issuing_trait::IssuingTrait;
pub use mates_trait::MatesTrait;
pub use minions_trait::MinionsTrait;
pub use recv_interaction_trait::RecvInteractionTrait;
pub use recv_request_trait::RecvRequestTrait;
pub use recv_verification_trait::RecvVerificationTrait;
pub use req_interaction_trait::ReqInteractionTrait;
pub use req_request_trait::ReqRequestTrait;
pub use req_vc_trait::ReqVcTrait;
pub use req_verification_trait::ReqVerificationTrait;
pub use token_requirements_trait::TokenRequirementsTrait;
pub use vc_request_trait::VcRequestTrait;
