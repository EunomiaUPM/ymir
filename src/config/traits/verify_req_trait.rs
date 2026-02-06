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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::config::types::VerifyReqConfig;
use crate::types::vcs::VcType;

pub trait VerifyReqConfigTrait {
    fn verify_req_config(&self) -> &VerifyReqConfig;
    fn is_cert_allowed(&self) -> bool {
        self.verify_req_config().is_cert_allowed
    }
    fn get_requested_vcs(&self) -> &[VcType] {
        &self.verify_req_config().vcs_requested
    }
}
