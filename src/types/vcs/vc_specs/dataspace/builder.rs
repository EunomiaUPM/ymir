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

use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use super::DataSpaceParticipant;
use crate::types::present::{Missing, Present};

#[derive(Serialize, Deserialize, Debug)]
pub struct DataSpaceParticipantBuilder<T> {
    pub id: Option<String>,
    pub nickname: String,
    pub dataspace_id: String,
    #[serde(skip)]
    _marker: PhantomData<T>
}

impl DataSpaceParticipantBuilder<Missing> {
    pub fn new(nickname: String, dataspace_id: String) -> Self {
        DataSpaceParticipantBuilder { id: None, nickname, dataspace_id, _marker: PhantomData }
    }
}

impl<T> DataSpaceParticipantBuilder<T> {
    pub fn id(self, id: impl Into<String>) -> DataSpaceParticipantBuilder<Present> {
        DataSpaceParticipantBuilder {
            id: Some(id.into()),
            nickname: self.nickname,
            dataspace_id: self.dataspace_id,
            _marker: PhantomData
        }
    }
}

impl DataSpaceParticipantBuilder<Present> {
    pub fn build(self) -> DataSpaceParticipant {
        DataSpaceParticipant {
            id: self.id.unwrap(),
            nickname: self.nickname,
            dataspace_id: self.dataspace_id
        }
    }
}
