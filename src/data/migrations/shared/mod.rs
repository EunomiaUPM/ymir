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

pub mod m20260622_120000_participant;
pub mod m20260622_120001_resource_req;
pub mod m20260622_120002_issuance;

// Short aliases — consumers pick the ones they need.
pub use m20260622_120000_participant as participant;
pub use m20260622_120001_resource_req as resource_req;
pub use m20260622_120002_issuance as issuance;
