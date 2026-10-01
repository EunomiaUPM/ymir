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

//! Keyset pagination, substring and role-path matching shared by the Postgres repositories.

use sea_orm::sea_query::{Condition, Expr, LikeExpr, SimpleExpr};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};

use crate::errors::{Errors, Outcome};
use crate::types::listing::{Keyset, ListPage, Listed, SortDirection};
use crate::types::roles::RolePath;

/// Runs a filtered listing as one page in the database, never in memory.
pub struct KeysetPager;

impl KeysetPager {
    /// Counts the rows matching `select`, then fetches the page ordered by `(timestamp, id)`
    /// that starts right after `page.after`.
    pub async fn fetch<E, S>(
        db: &DatabaseConnection,
        select: Select<E>,
        timestamp: SimpleExpr,
        id: E::Column,
        page: &ListPage<S>,
    ) -> Outcome<Listed<E::Model>>
    where
        E: EntityTrait,
        E::Model: Sync,
    {
        let total = select
            .clone()
            .count(db)
            .await
            .map_err(|e| Errors::db("Unable to count listing", Some(Box::new(e))))?;

        let order = match page.direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };
        let select = match &page.after {
            Some(after) => select.filter(Self::after(&timestamp, id, page.direction, after)),
            None => select,
        };
        let items = select
            .order_by(timestamp, order.clone())
            .order_by(id, order)
            .limit(page.limit)
            .all(db)
            .await
            .map_err(|e| Errors::db("Unable to fetch listing page", Some(Box::new(e))))?;
        Ok(Listed { items, total })
    }

    /// Rows strictly past the keyset in the listing's direction.
    fn after<C: ColumnTrait>(
        timestamp: &SimpleExpr,
        id: C,
        direction: SortDirection,
        after: &Keyset,
    ) -> Condition {
        let ts = || Expr::expr(timestamp.clone());
        let past_ts = match direction {
            SortDirection::Asc => ts().gt(after.timestamp),
            SortDirection::Desc => ts().lt(after.timestamp),
        };
        match &after.id {
            None => Condition::all().add(past_ts),
            Some(last_id) => {
                let past_id = match direction {
                    SortDirection::Asc => id.gt(last_id.clone()),
                    SortDirection::Desc => id.lt(last_id.clone()),
                };
                Condition::any()
                    .add(past_ts)
                    .add(Condition::all().add(ts().eq(after.timestamp)).add(past_id))
            }
        }
    }

    /// `LIKE` pattern matching `text` anywhere, with its wildcards taken literally.
    pub fn contains(text: &str) -> LikeExpr {
        LikeExpr::new(format!("%{}%", Self::escape(text))).escape('\\')
    }

    /// `LIKE` pattern matching every role path strictly below `role`; never `role` itself.
    pub fn below(role: &RolePath) -> LikeExpr {
        LikeExpr::new(format!("{}/%", Self::escape(role.as_str()))).escape('\\')
    }

    /// Escapes the `LIKE` wildcards in `text` so they match literally.
    fn escape(text: &str) -> String {
        text.replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    }
}
