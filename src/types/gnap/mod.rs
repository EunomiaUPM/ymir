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

pub mod access_token;
mod callback;
mod continue_request;
pub mod grant_request;
pub mod grant_response;
mod status;
mod vc_decision_approval;

pub use callback::{ApprovedCallbackBody, CallbackBody, RejectedCallbackBody};
pub use continue_request::ContinueRequest;
pub use status::GrantStatus;
pub use vc_decision_approval::VcDecisionApproval;

pub enum InteractionFinishResponse {
    Success(Option<String>),
    Failure(Option<String>),
}
