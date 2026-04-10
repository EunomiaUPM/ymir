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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DataSpaceParticipant {
    pub id: String,
    pub nickname: String,
    pub dataspace_id: String,
}

impl DataSpaceParticipant {
    pub fn new(
        id: impl Into<String>,
        nick: impl Into<String>,
        dataspace_id: impl Into<String>,
    ) -> DataSpaceParticipant {
        DataSpaceParticipant {
            id: id.into(),
            nickname: nick.into(),
            dataspace_id: dataspace_id.into(),
        }
    }
}
