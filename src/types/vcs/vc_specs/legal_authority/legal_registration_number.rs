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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LegalRegistrationNumberCredSubj {
    pub id: String,
    pub r#type: String,
    #[serde(rename = "gx:taxID", skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    #[serde(rename = "gx:EUID", skip_serializing_if = "Option::is_none")]
    pub euid: Option<String>,
    #[serde(rename = "gx:EORI", skip_serializing_if = "Option::is_none")]
    pub eori: Option<String>,
    #[serde(rename = "gx:vatID", skip_serializing_if = "Option::is_none")]
    pub vat_id: Option<String>,
    #[serde(rename = "gx:leiCode", skip_serializing_if = "Option::is_none")]
    pub lei_code: Option<String>
}

impl Default for LegalRegistrationNumberCredSubj {
    fn default() -> LegalRegistrationNumberCredSubj {
        LegalRegistrationNumberCredSubj {
            id: "".to_string(),
            r#type: "".to_string(),
            tax_id: None,
            euid: None,
            eori: None,
            vat_id: None,
            lei_code: None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegalRegistrationNumberTypes {
    TaxId,
    Euid,
    Eori,
    VatId,
    LeiCode
}
