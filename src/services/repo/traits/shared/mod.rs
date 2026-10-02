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

mod issuance_trait;
mod participant_relation_trait;
mod participant_trait;
mod resource_req_trait;

pub use issuance_trait::IssuanceRepoTrait;
#[cfg(feature = "mock")]
pub use issuance_trait::MockIssuanceRepoTrait;
#[cfg(feature = "mock")]
pub use participant_relation_trait::MockParticipantRelationRepoTrait;
pub use participant_relation_trait::ParticipantRelationRepoTrait;
#[cfg(feature = "mock")]
pub use participant_trait::MockParticipantRepoTrait;
pub use participant_trait::ParticipantRepoTrait;
#[cfg(feature = "mock")]
pub use resource_req_trait::MockResourceReqRepoTrait;
pub use resource_req_trait::ResourceReqRepoTrait;
