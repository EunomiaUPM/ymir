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

use crate::data::entities::wallet::vc;
use crate::errors::{Errors, Outcome};
use crate::services::repo::postgres::BasicPostgresRepo;
use crate::services::repo::traits::wallet::VcRepoTrait;
use crate::types::vcs::{InputDescriptor, VcType};
use async_trait::async_trait;
use sea_orm::prelude::Expr;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};

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
            .map_err(|e| Errors::db("Unable to get participant by type", Some(Box::new(e))))
    }
    async fn filter_by_desc(&self, input_descriptor: &InputDescriptor) -> Outcome<Vec<vc::Model>> {
        let mut condition = Condition::all();

        for field in &input_descriptor.constraints.fields {
            if let Some(json_path) = field.path.first() {
                let pattern = &field.filter.pattern;

                // Traducimos el JSONPath a la sintaxis que entiende Postgres para JSONB.
                // `->>` extrae como texto; `->` navega manteniendo el tipo JSONB.
                // Cuando el campo destino es un array JSON (ej. `type`), `->>` devuelve
                // la representación textual del array y el operador `~` de Postgres
                // puede buscar el patrón dentro de esa cadena.
                let pg_json_accessor = match json_path.as_str() {
                    "$.vc.type" => r#""parsed_document" -> 'vc' ->> 'type'"#.to_string(),
                    "$.type" => r#""parsed_document" ->> 'type'"#.to_string(),
                    // Fallback genérico para otros campos (ej. $.credentialSubject.id)
                    _ => {
                        let segments: Vec<&str> =
                            json_path.trim_start_matches("$.").split('.').collect();
                        match segments.as_slice() {
                            [] => continue,
                            [single] => {
                                format!(r#""parsed_document" ->> '{}'"#, single)
                            }
                            [head @ .., last] => {
                                let nav = head
                                    .iter()
                                    .map(|s| format!("-> '{}'", s))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                format!(r#""parsed_document" {} ->> '{}'"#, nav, last)
                            }
                        }
                    }
                };

                // Usamos el operador `~` de Postgres para evaluar el patrón como
                // expresión regular, que es el comportamiento estándar del campo
                // `pattern` en la especificación DIF Presentation Exchange.
                let sql_expr = format!("{} ~ $1", pg_json_accessor);

                condition = condition.add(Expr::cust_with_values(
                    sql_expr,
                    [sea_orm::Value::from(pattern.as_str())],
                ));
            }
        }

        vc::Entity::find()
            .filter(condition)
            .all(self.db())
            .await
            .map_err(|e| {
                Errors::db(
                    "Unable to filter VCs by input descriptor",
                    Some(Box::new(e)),
                )
            })
    }
}
