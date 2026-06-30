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

use sea_orm_migration::MigrationTrait;

pub mod m20260622_120020_grant;
pub mod m20260622_120021_interaction;
pub mod m20260622_120022_verification;

/// All sent-side migrations, executed together.
pub fn get_sent_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260622_120020_grant::Migration),
        Box::new(m20260622_120021_interaction::Migration),
        Box::new(m20260622_120022_verification::Migration),
    ]
}
