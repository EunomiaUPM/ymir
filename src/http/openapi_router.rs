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

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::get;

/// HTTP API Gateway Router governing self-contained API documentation assets.
///
/// Serves the standardized OpenAPI 3.0/3.1 specification metadata alongside a 
/// browser-rendered Swagger UI shell, ensuring machine-readable discovery and interactive 
/// contract validation for external ecosystem actors.
pub struct OpenapiRouter {
    /// In-memory raw representation of the compiled OpenAPI json contract sheet.
    openapi: String,
}

impl OpenapiRouter {
    /// Instantiates a new OpenAPI documentation service layer with its backing contract string.
    pub fn new(openapi: String) -> OpenapiRouter {
        OpenapiRouter { openapi }
    }

    /// Provisions and registers target endpoints into a unified sub-routing architecture branch.
    ///
    /// # Exposed Map
    /// * `GET /openapi.json` - Yields the raw machine-parsable JSON specification.
    /// * `GET /openapi`      - Renders an interactive Swagger UI documentation dashboard.
    pub fn router(self) -> Router {
        Router::new()
            .route("/openapi.json", get(Self::get_json))
            .route("/openapi", get(|| Self::get_swagger("openapi.json")))
            .with_state(self.openapi)
    }

    /// Axum endpoint handler retrieving the inner raw string asset forced as a structured JSON payload.
    async fn get_json(State(openapi): State<String>) -> impl IntoResponse {
        (
            StatusCode::OK,
            [("Content-Type", "application/json")],
            openapi.as_bytes().to_owned(),
        )
    }

    /// Utility helper injecting the destination document route parameter inside an embedded HTML container framework.
    async fn get_swagger(route: &str) -> impl IntoResponse {
        let html = format!(
            r#"<!doctype html>
            <html>
            <head>
              <meta charset="utf-8" />
              <title>Swagger UI</title>
              <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist/swagger-ui.css" />
              <style>
                body {{ margin:0; display:flex; flex-direction:column; height:100vh; }}
                #swagger-ui {{ flex:1; }}
                footer {{
                  text-align:center;
                  padding:10px;
                  background:#f8f8f8;
                  border-top:1px solid #ddd;
                  font-size:0.9em;
                  color:#555;
                }}
              </style>
            </head>
            <body>
              <div id="swagger-ui"></div>
              <footer>
                &copy; 2025 Universidad Politécnica de Madrid - UPM | Rainbow API Documentation
              </footer>
              <script src="https://unpkg.com/swagger-ui-dist/swagger-ui-bundle.js"></script>
              <script>
                window.onload = function() {{
                  SwaggerUIBundle({{
                    url: "{}",
                    dom_id: '#swagger-ui',
                  }});
                }};
              </script>
            </body>
            </html>"#,
            route
        );

        Html(html)
    }
}