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
                    .table(Issuance::Table)
                    .col(
                        ColumnDef::new(Issuance::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Issuance::SubjectName).string().not_null())
                    .col(ColumnDef::new(Issuance::PreAuthCode).string().not_null())
                    .col(
                        ColumnDef::new(Issuance::VcTypeConfig)
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Issuance::Token).string().not_null())
                    .col(
                        ColumnDef::new(Issuance::TokenExpiration)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Issuance::Nonce).string().not_null())
                    .col(ColumnDef::new(Issuance::Aud).string().not_null())
                    .col(ColumnDef::new(Issuance::IssuerDid).string().not_null())
                    .col(ColumnDef::new(Issuance::CredentialId).string().not_null())
                    .col(ColumnDef::new(Issuance::Credential).string())
                    .col(ColumnDef::new(Issuance::BuildCtx).json_binary().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Issuance::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Issuance {
    #[iden = "issuance"]
    Table,
    Id,
    SubjectName,
    PreAuthCode,
    VcTypeConfig,
    Token,
    TokenExpiration,
    Nonce,
    Aud,
    IssuerDid,
    CredentialId,
    Credential,
    BuildCtx,
}
