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
pub struct LegalRegistrationNumberCredSubj {
    #[serde(flatten)]
    pub base: BaseCredentialSubject,
    #[serde(rename = "gx:taxID", skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    #[serde(rename = "gx:EUID", skip_serializing_if = "Option::is_none")]
    pub euid: Option<String>,
    #[serde(rename = "gx:EORI", skip_serializing_if = "Option::is_none")]
    pub eori: Option<String>,
    #[serde(rename = "gx:vatID", skip_serializing_if = "Option::is_none")]
    pub vat_id: Option<String>,
    #[serde(rename = "gx:leiCode", skip_serializing_if = "Option::is_none")]
    pub lei_code: Option<String>,
}

impl LegalRegistrationNumberCredSubj {
    pub fn new<S: Into<String>, T: Into<String>>(
        model: LegalRegistrationNumberTypes,
        id: S,
        data: T,
    ) -> LegalRegistrationNumberCredSubj {
        let mut tax_id: Option<String> = None;
        let mut euid: Option<String> = None;
        let mut eori: Option<String> = None;
        let mut vat_id: Option<String> = None;
        let mut lei_code: Option<String> = None;

        let r#type = match model {
            LegalRegistrationNumberTypes::TaxId => {
                tax_id = Some(data.into());
                "gx:taxID"
            }
            LegalRegistrationNumberTypes::Euid => {
                euid = Some(data.into());
                "gx:EUID"
            }
            LegalRegistrationNumberTypes::Eori => {
                eori = Some(data.into());
                "gx:EORI"
            }
            LegalRegistrationNumberTypes::VatId => {
                vat_id = Some(data.into());
                "gx:vatID"
            }
            LegalRegistrationNumberTypes::LeiCode => {
                lei_code = Some(data.into());
                "gx:leiCode"
            }
        }
        .to_string();

        LegalRegistrationNumberCredSubj {
            base: BaseCredentialSubject { id: id.into(), r#type },
            tax_id,
            euid,
            eori,
            vat_id,
            lei_code,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegalRegistrationNumberTypes {
    TaxId,
    Euid,
    Eori,
    VatId,
    LeiCode,
}
