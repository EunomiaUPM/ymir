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

use crate::impl_serde_via_str;
use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// A W3C Data Integrity proof purpose (`https://www.w3.org/TR/vc-data-integrity/#proof-purposes`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofPurpose {
    AssertionMethod,
    Authentication,
    KeyAgreement,
    CapabilityInvocation,
    CapabilityDelegation,
    Other(String),
}

/// The proof purpose that proofs over this document must declare.
pub trait HasProofPurpose {
    const PURPOSE: ProofPurpose;
}

impl Display for ProofPurpose {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ProofPurpose::AssertionMethod => "assertionMethod",
            ProofPurpose::Authentication => "authentication",
            ProofPurpose::KeyAgreement => "keyAgreement",
            ProofPurpose::CapabilityInvocation => "capabilityInvocation",
            ProofPurpose::CapabilityDelegation => "capabilityDelegation",
            ProofPurpose::Other(s) => s.as_str(),
        };
        write!(f, "{s}")
    }
}

impl FromStr for ProofPurpose {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "assertionMethod" => ProofPurpose::AssertionMethod,
            "authentication" => ProofPurpose::Authentication,
            "keyAgreement" => ProofPurpose::KeyAgreement,
            "capabilityInvocation" => ProofPurpose::CapabilityInvocation,
            "capabilityDelegation" => ProofPurpose::CapabilityDelegation,
            other => ProofPurpose::Other(other.to_string()),
        })
    }
}

impl_serde_via_str!(ProofPurpose);
