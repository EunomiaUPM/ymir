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

//! The access rules over the role tree, for anything that holds a role.
//!
//! A type implements only its base, [`RoleTrait::role`] (and [`UserTrait::id`] if it is a user),
//! and gets every comparison and check from the traits:
//! - **handles** a record with only a role: the same role, one below it, or any for the root;
//! - **reaches** a record with an author: its own, or created under a role below its own (users
//!   sharing a role do not reach each other's), or any for the root;
//! - **sees** a record it handles or reaches, or one whose [`Visibility`] opens it to everyone.
//!
//! Handling or reaching lets the caller read **and act on** the record; seeing only lets it
//! read. An unreachable record looks like a missing one.

use crate::errors::{Errors, Outcome};
use crate::types::oauth::RolePath;
use crate::types::participants::Visibility;

/// Something placed in the role tree: implement [`role`](Self::role) and get the comparisons
/// over the tree and the checks over records assigned to a role.
pub trait RoleTrait {
    /// The role it acts under.
    fn role(&self) -> &RolePath;

    // ==========================================================================================
    // Comparisons over the tree
    // ==========================================================================================

    /// Whether it is the root [`RolePath::root`].
    fn is_root(&self) -> bool {
        self.role().as_str() == RolePath::ROOT
    }

    /// Whether its role hangs strictly below `other`. A role is not below itself, and siblings
    /// are not below each other.
    fn is_below(&self, other: &RolePath) -> bool {
        self.role()
            .as_str()
            .starts_with(&format!("{}/", other.as_str()))
    }

    /// Whether `other` hangs strictly below its role.
    fn is_above(&self, other: &RolePath) -> bool {
        other.is_below(self.role())
    }

    // ==========================================================================================
    // Checks over records assigned to a role
    // ==========================================================================================

    /// Fails with a forbidden error unless it is the root.
    fn require_root(&self) -> Outcome<()> {
        match self.is_root() {
            true => Ok(()),
            false => Err(Errors::forbidden("Only admins can perform this action", None)),
        }
    }

    /// Whether it handles a record assigned to `role`: the same role, one below it, or any for
    /// the root.
    fn handles(&self, role: &RolePath) -> bool {
        self.is_root() || role == self.role() || self.is_above(role)
    }

    /// Fails unless it handles the record; an unhandled record looks like a missing one.
    fn ensure_handles(&self, role: &RolePath, id: &str) -> Outcome<()> {
        if self.handles(role) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }

    /// Whether it sees a record assigned to `role`: it handles it, or it is `Public`.
    fn sees_team(&self, role: &RolePath, visibility: &Visibility) -> bool {
        *visibility == Visibility::Public || self.handles(role)
    }

    /// Fails unless it [sees](Self::sees_team) the record, which then looks like a missing one.
    fn ensure_sees_team(&self, role: &RolePath, visibility: &Visibility, id: &str) -> Outcome<()> {
        if self.sees_team(role, visibility) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }
}

/// Someone with an identity in the role tree: implement [`id`](Self::id) (plus
/// [`RoleTrait::role`]) and get the checks over records with an author.
pub trait UserTrait: RoleTrait {
    /// Who it is.
    fn id(&self) -> &str;

    /// Whether it reaches a record created by `owner_id` under `owner_role`: its own, one
    /// created under a role below its own, or any for the root. Not one of another user of its
    /// same role.
    fn reaches(&self, owner_id: &str, owner_role: &RolePath) -> bool {
        self.is_root() || owner_id == self.id() || self.is_above(owner_role)
    }

    /// Fails unless it reaches the record; an unreachable record looks like a missing one.
    fn ensure_reaches(&self, owner_id: &str, owner_role: &RolePath, id: &str) -> Outcome<()> {
        if self.reaches(owner_id, owner_role) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }

    /// Whether it sees a record created by `owner_id` under `owner_role`: it reaches it, or its
    /// visibility is not `Private` (an `Anonymous` one is seen without its author).
    fn sees(&self, owner_id: &str, owner_role: &RolePath, visibility: &Visibility) -> bool {
        *visibility != Visibility::Private || self.reaches(owner_id, owner_role)
    }

    /// Fails unless it [sees](Self::sees) the record, which then looks like a missing one.
    fn ensure_sees(
        &self,
        owner_id: &str,
        owner_role: &RolePath,
        visibility: &Visibility,
        id: &str,
    ) -> Outcome<()> {
        if self.sees(owner_id, owner_role, visibility) {
            Ok(())
        } else {
            Err(Errors::missing_resource(id, "resource not found", None))
        }
    }
}
