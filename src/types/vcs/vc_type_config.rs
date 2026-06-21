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

use std::convert::Infallible;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use sea_orm::FromJsonQueryResult;
use crate::impl_serde_via_str;
use super::VcType;
use super::VcFormat;

#[derive(Debug, Clone, PartialEq, FromJsonQueryResult)]
pub struct VcTypeConfig {
    vc_type: VcType,
    format: VcFormat,
}

impl Display for VcTypeConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let vc_type = self.vc_type.to_string().replace(":", "_");
        write!(f, "{}_{}", vc_type, self.format)
    }
}

impl FromStr for VcTypeConfig {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let known_formats = [
            VcFormat::JwtVcJson,
            VcFormat::LdpVc,
            VcFormat::SdJwtVc,
            VcFormat::MsoMdoc,
        ];

        let (prefix, format) = known_formats
            .into_iter()
            .find_map(|f| {
                let suffix = format!("_{f}");
                s.strip_suffix(&suffix).map(|p| (p.to_string(), f))
            })
            .unwrap_or_else(|| {
                let (prefix, format_str) = s.rsplit_once('_').unwrap_or((s, ""));
                (prefix.to_string(), VcFormat::Other(format_str.to_string()))
            });

        let vc_type_str = prefix.replacen('_', ":", 1);
        let Ok(vc_type) = VcType::from_str(&vc_type_str);

        Ok(VcTypeConfig { vc_type, format })
    }
}

impl_serde_via_str!(VcTypeConfig);

impl VcTypeConfig {
    pub fn new(vc_type: VcType, format: VcFormat) -> Self {
        VcTypeConfig { vc_type, format }
    }
    pub fn supported() -> Vec<VcTypeConfig> {
        let mut configs = Vec::new();
        for vc_type in VcType::supported() {
            for format in VcFormat::supported() {
                configs.push(VcTypeConfig {
                    vc_type: vc_type.clone(),
                    format: format.clone(),
                });
            }
        }
        configs
    }
    pub fn is_supported(&self) -> bool {
        if Self::supported().contains(self) { true } else { false }
    }
    pub fn vc_type(&self) -> &VcType {
        &self.vc_type
    }
    pub fn format(&self) -> &VcFormat {
        &self.format
    }
}