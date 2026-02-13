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

use crate::types::vcs::vc_specs::BaseCredentialSubject;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DataSpaceParticipant {
    #[serde(flatten)]
    pub base: BaseCredentialSubject,
    pub dataspace_id: String,
}

impl DataSpaceParticipant {
    pub fn new<R: Into<String>, S: Into<String>>(id: R, dataspace_id: S) -> DataSpaceParticipant {
        DataSpaceParticipant {
            base: BaseCredentialSubject {
                id: id.into(),
                r#type: "DataspaceParticipant".to_string(),
            },
            dataspace_id: dataspace_id.into(),
        }
    }
}
