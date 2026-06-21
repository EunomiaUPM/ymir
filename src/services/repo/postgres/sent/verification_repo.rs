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

use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

use crate::data::entities::sent::verification;
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::sent::SentVerificationRepoTrait;

pub struct SentVerificationPostgresRepo {
    db: DatabaseConnection,
}

impl SentVerificationPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for SentVerificationPostgresRepo {
    type Entity = verification::Entity;
    type Plan = verification::Plan;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl SentVerificationRepoTrait for SentVerificationPostgresRepo {
    // async fn end(&self, id: &str) -> Outcome<()> {
    //     let model = self.get_by_id(id).await?;
    //     let mut active: verification::ActiveModel = model.into();
    //     active.ended_at = ActiveValue::Set(Some(Utc::now()));
    //     active.status = ActiveValue::Set(VerificationStatus::Completed);
    //     active
    //         .update(self.db())
    //         .await
    //         .map_err(|e| Errors::db("Unable to end verification", Some(Box::new(e))))?;
    //     Ok(())
    // }
}
