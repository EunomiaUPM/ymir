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

use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::{BadFormat, Errors, Outcome};

pub const BITSTRING_STATUS_LIST_ENTRY: &str = "BitstringStatusListEntry";
pub const BITSTRING_STATUS_LIST: &str = "BitstringStatusList";
pub const BITSTRING_STATUS_LIST_CREDENTIAL: &str = "BitstringStatusListCredential";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusPurpose {
    Revocation,
    Suspension,
    Refresh,
    Message,
}

impl Display for StatusPurpose {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            StatusPurpose::Revocation => "revocation",
            StatusPurpose::Suspension => "suspension",
            StatusPurpose::Refresh => "refresh",
            StatusPurpose::Message => "message",
        };
        write!(f, "{s}")
    }
}

impl FromStr for StatusPurpose {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "revocation" => Ok(StatusPurpose::Revocation),
            "suspension" => Ok(StatusPurpose::Suspension),
            "refresh" => Ok(StatusPurpose::Refresh),
            "message" => Ok(StatusPurpose::Message),
            other => Err(Errors::parse(format!("Invalid status purpose '{other}'"), None)),
        }
    }
}

/// `credentialStatus` of a VC: a Bitstring Status List entry, or any other status mechanism.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum VcStatus {
    Bitstring(BitstringStatusListEntry),
    Other(Value),
}

impl VcStatus {
    pub fn as_bitstring(&self) -> Option<&BitstringStatusListEntry> {
        match self {
            VcStatus::Bitstring(entry) if entry.r#type == BITSTRING_STATUS_LIST_ENTRY => Some(entry),
            _ => None,
        }
    }
}

impl From<BitstringStatusListEntry> for VcStatus {
    fn from(entry: BitstringStatusListEntry) -> Self {
        VcStatus::Bitstring(entry)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BitstringStatusListEntry {
    pub id: String,
    pub r#type: String,
    #[serde(rename = "statusPurpose")]
    pub status_purpose: StatusPurpose,
    #[serde(rename = "statusListIndex")]
    pub status_list_index: String,
    #[serde(rename = "statusListCredential")]
    pub status_list_credential: String,
}

impl BitstringStatusListEntry {
    pub fn new(list_url: &str, purpose: StatusPurpose, index: u64) -> Self {
        Self {
            id: format!("{list_url}#{index}"),
            r#type: BITSTRING_STATUS_LIST_ENTRY.to_string(),
            status_purpose: purpose,
            status_list_index: index.to_string(),
            status_list_credential: list_url.to_string(),
        }
    }

    pub fn index(&self) -> Outcome<usize> {
        self.status_list_index.parse().map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "statusListIndex is not a non-negative integer",
                Some(Box::new(e)),
            )
        })
    }
}

/// `credentialSubject` of a `BitstringStatusListCredential`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BitstringStatusList {
    pub id: String,
    pub r#type: String,
    #[serde(rename = "statusPurpose")]
    pub status_purpose: StatusPurpose,
    #[serde(rename = "encodedList")]
    pub encoded_list: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
}

impl BitstringStatusList {
    pub fn new(list_url: &str, purpose: StatusPurpose, encoded_list: String, ttl: Option<u64>) -> Self {
        Self {
            id: format!("{list_url}#list"),
            r#type: BITSTRING_STATUS_LIST.to_string(),
            status_purpose: purpose,
            encoded_list,
            ttl,
        }
    }
}
