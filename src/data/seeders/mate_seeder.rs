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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::data::entities::mates;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

pub struct MateSeeder;

impl MateSeeder {
    pub async fn seed(db: &DatabaseConnection, did: String, url: String) -> anyhow::Result<()> {
        let exists = mates::Entity::find_by_id(&did).one(db).await?.is_some();

        if exists {
            return Ok(());
        }

        mates::ActiveModel {
            participant_id: ActiveValue::Set(did),
            participant_slug: ActiveValue::Set("Myself".to_string()),
            participant_type: ActiveValue::Set("Authority".to_string()),
            base_url: ActiveValue::Set(url),
            token: ActiveValue::Set(None),
            saved_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            last_interaction: ActiveValue::Set(chrono::Utc::now().naive_utc()),
            is_me: ActiveValue::Set(true),
        }
        .insert(db)
        .await?;
        Ok(())
    }
}
