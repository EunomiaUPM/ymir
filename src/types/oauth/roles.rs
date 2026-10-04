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

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::errors::{BadFormat, Errors};
use crate::{impl_seaorm_via_str, impl_serde_via_str};

/// Role of a user, written as its full path in the organisation's role tree
/// (e.g. `/admin/upm/dit`). A shorter path sits higher in the tree; the tree hangs from
/// [`RolePath::root`].
///
/// Only valid paths can be built: they start with `/`, do not end with `/` and have no empty
/// segments. An empty or malformed role therefore never turns into a prefix that matches every
/// other role.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RolePath(String);

impl RolePath {
    const ROOT: &'static str = "/admin";

    /// The superuser role `/admin`, from which every other role hangs.
    pub fn root() -> Self {
        Self(Self::ROOT.to_string())
    }

    /// The path as stored and compared.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_root(&self) -> bool {
        self.0 == Self::ROOT
    }

    /// Whether `self` hangs strictly below `other` in the tree. A path is not below itself,
    /// and siblings are not below each other.
    pub fn is_below(&self, other: &RolePath) -> bool {
        self.0.starts_with(&format!("{}/", other.0))
    }
}

impl Display for RolePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for RolePath {
    type Err = Errors;

    /// Fails with a format error when `s` is not a valid role path.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let valid = s.starts_with('/')
            && !s.ends_with('/')
            && s[1..].split('/').all(|segment| !segment.trim().is_empty());

        if !valid {
            return Err(Errors::format(
                BadFormat::Received,
                format!("invalid role path '{s}'"),
                None,
            ));
        }
        Ok(Self(s.to_string()))
    }
}

impl_serde_via_str!(RolePath);
impl_seaorm_via_str!(RolePath, 255);
