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
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm::prelude::Expr;
use crate::data::entities::wallet::vc;
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::{BasicPostgresRepo};
use crate::services::repo::traits::wallet::VcRepoTrait;
use crate::types::vcs::{InputDescriptor, VcType};

pub struct VcPostgresRepo {
    db: DatabaseConnection,
}

impl VcPostgresRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BasicPostgresRepo for VcPostgresRepo {
    type Entity = vc::Entity;
    type Plan = vc::Model;

    fn db(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl VcRepoTrait for VcPostgresRepo {
    async fn filter_by_type(&self, r#type: VcType) -> Outcome<Vec<vc::Model>> {
        vc::Entity::find()
            .filter(vc::Column::VcType.eq(r#type))
            .all(self.db())
            .await
            .map_err(|e| Errors::db(
                "Unable to get participant by type",
                Some(Box::new(e)),
            ))
    }
    async fn filter_by_desc(&self, input_descriptor: &InputDescriptor) -> Outcome<Vec<vc::Model>> {
        todo!()
        // let mut condition = Condition::all();
        //
        // for field in &input_descriptor.constraints.fields {
        //     if let Some(json_path) = field.path.first() {
        //         let pattern = &field.filter.pattern;
        //
        //         // Traducimos el JSONPath a la sintaxis que entiende Postgres para JSONB
        //         // "$.vc.type" -> 'vc' ->> 'type'
        //         // "$.type"    ->> 'type'
        //         let pg_json_accessor = match json_path.as_str() {
        //             "$.vc.type" => r#""parsed_document" -> 'vc' ->> 'type'"#.to_string(),
        //             "$.type" => r#""parsed_document" ->> 'type'"#.to_string(),
        //             // Fallback genérico por si vienen otros campos (ej. $.credentialSubject.id)
        //             _ => {
        //                 // Limpieza rápida del "$." inicial para convertir paths simples
        //                 let clean_path = json_path.trim_start_matches("$.").replace('.', "' -> '");
        //                 // El último elemento debe extraerse como texto (->>)
        //                 if let Some(last_dot) = clean_path.rfind("' -> '") {
        //                     let (head, tail) = clean_path.split_at(last_dot);
        //                     let tail_cleaned = tail.trim_start_matches("' -> '");
        //                     format!(r#""parsed_document" -> '{}' ->> '{}'"#, head, tail_cleaned)
        //                 } else {
        //                     format!(r#""parsed_document" ->> '{}'"#, clean_path)
        //                 }
        //             }
        //         };
        //
        //         // Si el patrón es una expresión regular o un tipo exacto, aplicamos el operador de Postgres.
        //         // Usamos '~' para evaluar expresiones regulares en Postgres (comportamiento estándar de 'pattern' en Present Exchange)
        //         let sql_inject = format!("{} ~ ?", pg_json_accessor);
        //
        //         condition = condition.add(Expr::cust_with_values(sql_inject, vec![pattern.into()]));
        //     }
        // }
        //
        // vc::Entity::find()
        //     .filter(condition)
        //     .all(self.db())
        //     .await.map_err(|e| Errors::db(
        //     "Unable to get vc by filter",
        //     Some(Box::new(e)),
        // ))
    }
}