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

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
ALTER TABLE recv_interactions ADD COLUMN grant_id varchar NOT NULL DEFAULT '';
UPDATE recv_interactions SET grant_id = id;
ALTER TABLE recv_interactions ALTER COLUMN grant_id DROP DEFAULT;
ALTER TABLE recv_interactions ADD COLUMN created_at timestamptz NOT NULL DEFAULT now();
CREATE INDEX idx_recv_interactions_grant_id ON recv_interactions (grant_id);
ALTER TABLE recv_verification ADD COLUMN grant_id varchar NOT NULL DEFAULT '';
UPDATE recv_verification SET grant_id = id;
ALTER TABLE recv_verification ALTER COLUMN grant_id DROP DEFAULT;
CREATE INDEX idx_recv_verification_grant_id ON recv_verification (grant_id);
UPDATE recv_grants SET status = 'Finalized', ended_at = now()
    WHERE kind = 'AccessToken' AND status IN ('Pending', 'Processing', 'Approved');
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DROP INDEX IF EXISTS idx_recv_verification_grant_id;
ALTER TABLE recv_verification DROP COLUMN grant_id;
DROP INDEX IF EXISTS idx_recv_interactions_grant_id;
ALTER TABLE recv_interactions DROP COLUMN created_at;
ALTER TABLE recv_interactions DROP COLUMN grant_id;
"#,
            )
            .await?;
        Ok(())
    }
}
