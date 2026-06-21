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
                    .table(SentVerifications::Table)
                    .col(
                        ColumnDef::new(SentVerifications::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SentVerifications::Uri).string().not_null())
                    .col(ColumnDef::new(SentVerifications::Scheme).string().not_null())
                    .col(ColumnDef::new(SentVerifications::ResponseType).string().not_null())
                    .col(ColumnDef::new(SentVerifications::ClientId).string().not_null())
                    .col(ColumnDef::new(SentVerifications::ResponseMode).string().not_null())
                    .col(ColumnDef::new(SentVerifications::PdUri).string().not_null())
                    .col(ColumnDef::new(SentVerifications::ClientIdScheme).string().not_null())
                    .col(ColumnDef::new(SentVerifications::Nonce).string().not_null())
                    .col(ColumnDef::new(SentVerifications::ResponseUri).string().not_null())
                    .col(ColumnDef::new(SentVerifications::Status).string().not_null())
                    .col(ColumnDef::new(SentVerifications::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(SentVerifications::EndedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SentVerifications::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum SentVerifications {
    #[iden = "sent_verifications"]
    Table,
    Id,
    Uri,
    Scheme,
    ResponseType,
    ClientId,
    ResponseMode,
    PdUri,
    ClientIdScheme,
    Nonce,
    ResponseUri,
    Status,
    CreatedAt,
    EndedAt,
}
