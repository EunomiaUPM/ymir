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

use super::m20261028_120000_registered_dataset::RegisteredDatasets;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Columns {
    Requirements,
    Offers,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RegisteredDatasets::Table)
                    .drop_column(Columns::Requirements)
                    .add_column(
                        ColumnDef::new(Columns::Offers)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RegisteredDatasets::Table)
                    .drop_column(Columns::Offers)
                    .add_column(
                        ColumnDef::new(Columns::Requirements)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'[]'::jsonb")),
                    )
                    .to_owned(),
            )
            .await
    }
}
