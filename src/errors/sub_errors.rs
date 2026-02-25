/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorInfo {
    pub message: String,
    pub error_code: u16,
    #[serde(skip)]
    pub status_code: StatusCode,
    pub details: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HttpContext {
    pub http_code: Option<u16>,
    pub url: String,
    pub method: String,
}

#[derive(Debug)]
pub enum PetitionFailure {
    Network,                 // reqwest::Error de conexión
    HttpStatus(StatusCode),  // 4xx/5xx del servidor remoto
    Deserialization(String), // raw_text del error de parseo
    Serialization,           // error preparando el body
    Concurrency,             // semáforo cerrado
    BodyRead,                // error leyendo el stream
}

impl Display for PetitionFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PetitionFailure::Network => write!(f, "Network failure"),
            PetitionFailure::HttpStatus(code) => write!(f, "Remote HTTP error {}", code),
            PetitionFailure::Deserialization(raw) => {
                write!(f, "Deserialization failed. Raw: {}", raw)
            }
            PetitionFailure::Serialization => write!(f, "Serialization failed"),
            PetitionFailure::BodyRead => write!(f, "Failed to read response body"),
            PetitionFailure::Concurrency => write!(f, "Concurrency limit reached"),
        }
    }
}
