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
use sea_orm::FromJsonQueryResult;
use std::convert::Infallible;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Hash, PartialEq, Eq, FromJsonQueryResult)]
pub enum VcType {
    Eori,
    Euid,
    LeiCode,
    LocalRegistrationNumber,
    TaxId,
    VatId,
    DataspaceParticipant,
    DataspacePremium,
    LegalPerson,
    /// `gx:Issuer`: accepts the Gaia-X terms and conditions.
    GxIssuer,
    GxLabel,
    /// `gx:ParticipantComplianceCredential`: the label without a level, read like [`VcType::GxLabel`].
    ParticipantCompliance,
    Other(String),
}

impl FromStr for VcType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "gx:eori" => Ok(VcType::Eori),
            "gx:euid" => Ok(VcType::Euid),
            "gx:leicode" => Ok(VcType::LeiCode),
            "gx:localregistrationnumber" => Ok(VcType::LocalRegistrationNumber),
            "gx:taxid" => Ok(VcType::TaxId),
            "gx:vatid" => Ok(VcType::VatId),
            "dataspaceparticipant" => Ok(VcType::DataspaceParticipant),
            "dataspacepremium" => Ok(VcType::DataspacePremium),
            "gx:legalperson" => Ok(VcType::LegalPerson),
            "gx:issuer" => Ok(VcType::GxIssuer),
            "gx:labelcredential" => Ok(VcType::GxLabel),
            "gx:participantcompliancecredential" => Ok(VcType::ParticipantCompliance),
            other => Ok(VcType::Other(other.to_string())),
        }
    }
}

impl Display for VcType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            VcType::Eori => "gx:EORI",
            VcType::Euid => "gx:EUID",
            VcType::LeiCode => "gx:LeiCode",
            VcType::LocalRegistrationNumber => "gx:LocalRegistrationNumber",
            VcType::TaxId => "gx:TaxID",
            VcType::VatId => "gx:VatID",
            VcType::DataspaceParticipant => "DataSpaceParticipant",
            VcType::DataspacePremium => "DataSpacePremium",
            VcType::LegalPerson => "gx:LegalPerson",
            VcType::GxIssuer => "gx:Issuer",
            VcType::GxLabel => "gx:LabelCredential",
            VcType::ParticipantCompliance => "gx:ParticipantComplianceCredential",
            VcType::Other(other) => other,
        };

        write!(f, "{s}")
    }
}

impl VcType {
    pub fn supported() -> Vec<VcType> {
        vec![
            VcType::Eori,
            VcType::Euid,
            VcType::LeiCode,
            VcType::LocalRegistrationNumber,
            VcType::TaxId,
            VcType::VatId,
            VcType::DataspaceParticipant,
            VcType::DataspacePremium,
            VcType::LegalPerson,
            VcType::GxIssuer,
            VcType::GxLabel,
            VcType::ParticipantCompliance,
        ]
    }
    pub fn is_legal_registration_number(&self) -> bool {
        matches!(
            self,
            VcType::VatId
                | VcType::LeiCode
                | VcType::TaxId
                | VcType::LocalRegistrationNumber
                | VcType::Eori
                | VcType::Euid
        )
    }
    /// Whether it is a Gaia-X type, which needs [`GAIA_X_CONTEXT`].
    pub fn is_gaia_x(&self) -> bool {
        self.to_string().starts_with("gx:")
    }
}
impl_serde_via_str!(VcType);

/// The `@context` of the Gaia-X ontology the `gx:` terms come from (2.5.0).
pub const GAIA_X_CONTEXT: &str = "https://w3id.org/gaia-x/2.5.0#";
