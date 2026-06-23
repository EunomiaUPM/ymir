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

use async_trait::async_trait;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::errors::{Errors, Outcome};
use crate::utils::{read_json, write_json, HasId};

#[async_trait]
pub trait BasicDiskTrait: Send + Sync + 'static {
    type Model: Serialize + DeserializeOwned + HasId + Send + Sync + 'static;
    type Plan: Into<Self::Model> + Send + Sync + 'static;
    fn dir(&self) -> &Path;

    async fn basic_get_all(&self) -> Outcome<Vec<Self::Model>> {
        let dir = self.dir();

        read_json(dir)
    }

    async fn basic_get_by_id(&self, id: &str) -> Outcome<Self::Model> {
        let path = self.path_for(id);
        if !path.exists() {
            return Err(Errors::missing_resource(
                id,
                format!("no entry with id {id}"),
                None,
            ));
        }
        read_json(&path)
    }

    async fn basic_create(&self, plan: Self::Plan) -> Outcome<Self::Model> {
        let model = plan.into();
        let path = self.path_for(model.id());
        if path.exists() {
            return Err(Errors::db(
                format!("entry with id {} already exists", model.id()),
                None,
            ));
        }
        write_json(&path, &model)?;
        Ok(model)
    }

    async fn basic_update(&self, model: Self::Model) -> Outcome<Self::Model> {
        let path = self.path_for(model.id());
        if !path.exists() {
            return Err(Errors::missing_resource(
                model.id(),
                format!("cannot update missing entry {}", model.id()),
                None,
            ));
        }
        write_json(&path, &model)?;
        Ok(model)
    }

    async fn basic_delete(&self, id: &str) -> Outcome<()> {
        let path = self.path_for(id);
        match std::fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) => Err(Errors::write(
                path.display().to_string(),
                "remove_file failed",
                Some(Box::new(e)),
            )),
        }
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.dir().join(format!("{id}.json"))
    }
}

// ========================================= BLANKET IMPL ==========================================
// NOTA! CAMBIAR A MACROS, RUST NO SOPORTA ESTO
// #[async_trait]
// impl<R> CrudRepoTrait<R::Model, R::Plan> for R
// where
//     R: BasicDiskTrait,
//     R::Model: Serialize + DeserializeOwned + HasId + Send + Sync + 'static,
//     R::Plan: Into<R::Model> + Send + Sync + 'static,
// {
//     async fn get_all(
//         &self,
//         _limit: Option<u64>,
//         _offset: Option<u64>,
//     ) -> Outcome<Vec<R::Model>> {
//         self.basic_get_all().await
//     }
//
//     async fn get_by_id(&self, id: &str) -> Outcome<R::Model> {
//         self.basic_get_by_id(id).await
//     }
//
//     async fn create(&self, plan: R::Plan) -> Outcome<R::Model> {
//         self.basic_create(plan).await
//     }
//
//     async fn update(
//         &self,
//         model: R::Model,
//     ) -> Outcome<R::Model> {
//         self.basic_update(model).await
//     }
//
//     async fn delete(&self, id: &str) -> Outcome<()> {
//         self.basic_delete(id).await
//     }
// }
