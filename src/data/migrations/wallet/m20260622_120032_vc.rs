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

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Vcs::Table)
                    .col(ColumnDef::new(Vcs::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Vcs::VcBody).json_binary().not_null())
                    .col(ColumnDef::new(Vcs::VcType).json_binary().not_null())
                    .col(ColumnDef::new(Vcs::VcFormat).json_binary().not_null())
                    .col(ColumnDef::new(Vcs::HolderDid).string().not_null())
                    .col(ColumnDef::new(Vcs::IssuerDid).string().not_null())
                    .col(ColumnDef::new(Vcs::ParsedDocument).json_binary().not_null())
                    .col(ColumnDef::new(Vcs::ValidUntil).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Vcs::AddedOn)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Vcs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Vcs {
    #[iden = "vcs"]
    Table,
    Id,
    VcBody,
    VcType,
    VcFormat,
    HolderDid,
    IssuerDid,
    ParsedDocument,
    ValidUntil,
    AddedOn,
}
