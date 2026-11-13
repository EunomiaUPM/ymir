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
ALTER TABLE resources_reqs
    DROP COLUMN type,
    DROP COLUMN actions,
    DROP COLUMN locations,
    DROP COLUMN datatypes,
    DROP COLUMN identifier,
    DROP COLUMN privileges,
    ADD COLUMN access jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN granted jsonb;
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
ALTER TABLE resources_reqs
    DROP COLUMN access,
    DROP COLUMN granted,
    ADD COLUMN type varchar(32) NOT NULL DEFAULT 'api-access',
    ADD COLUMN actions jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN locations text[],
    ADD COLUMN datatypes text[],
    ADD COLUMN identifier varchar,
    ADD COLUMN privileges text[];
"#,
            )
            .await?;
        Ok(())
    }
}
