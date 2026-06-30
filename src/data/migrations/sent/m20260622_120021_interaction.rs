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
                    .table(SentInteractions::Table)
                    .col(
                        ColumnDef::new(SentInteractions::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SentInteractions::Start)
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SentInteractions::Method).string_len(32).not_null())
                    .col(
                        ColumnDef::new(SentInteractions::CallbackUri)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SentInteractions::ClientNonce)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SentInteractions::HashMethod)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(SentInteractions::Hints).string())
                    .col(ColumnDef::new(SentInteractions::ContinueEndpoint).string())
                    .col(ColumnDef::new(SentInteractions::ContinueToken).string())
                    .col(ColumnDef::new(SentInteractions::ContinueWait).big_integer())
                    .col(ColumnDef::new(SentInteractions::AsNonce).string())
                    .col(ColumnDef::new(SentInteractions::OidcVpUri).string())
                    .col(ColumnDef::new(SentInteractions::InteractRef).string())
                    .col(ColumnDef::new(SentInteractions::Hash).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SentInteractions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum SentInteractions {
    #[iden = "sent_interactions"]
    Table,
    Id,
    Start,
    Method,
    CallbackUri,
    ClientNonce,
    HashMethod,
    Hints,
    ContinueEndpoint,
    ContinueToken,
    ContinueWait,
    AsNonce,
    OidcVpUri,
    InteractRef,
    Hash,
}
