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

//! Error constructors and their JSON body.

use axum::response::IntoResponse;
use ymir::errors::{ErrorReason, Errors};

fn reason(path: &str, code: u32, message: &str) -> ErrorReason {
    ErrorReason {
        path: path.into(),
        code,
        message: message.into(),
    }
}

/// A validation error keeps every reason and answers 400.
#[test]
fn validation_with_keeps_every_reason() {
    let err = Errors::validation_with(vec![
        reason("consumerPid", 2, "must be a valid URN"),
        reason("offer", 1, "field is required"),
    ]);
    assert_eq!(err.info().reasons.len(), 2);
    assert_eq!(err.info().status_code, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(
        err.reason(),
        "consumerPid: must be a valid URN; offer: field is required"
    );
}

/// The reasons reach the JSON body; errors without them keep the old shape.
#[tokio::test]
async fn reasons_are_serialized_only_when_present() {
    let with = Errors::validation_with(vec![reason("offer", 1, "field is required")]);
    let body = axum::body::to_bytes(with.into_response().into_body(), 4096)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["reasons"][0]["path"], "offer");
    assert_eq!(json["reasons"][0]["code"], 1);

    let without = Errors::parse("bad", None);
    let body = axum::body::to_bytes(without.into_response().into_body(), 4096)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("reasons").is_none());
}
