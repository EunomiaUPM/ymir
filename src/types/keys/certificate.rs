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

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use x509_parser::pem::parse_x509_pem;
use x509_parser::prelude::*;

use crate::errors::{Errors, Outcome};
use crate::types::keys::PublicKey;

pub struct Certificate {
    der: Vec<u8>,
}

impl Certificate {
    pub fn try_from_pem(cert_pem: &str) -> Outcome<Self> {
        let normalized = normalize_pem(cert_pem);
        let (_, pem) = parse_x509_pem(normalized.as_bytes())
            .map_err(|e| Errors::parse("Failed to parse certificate PEM", Some(Box::new(e))))?;

        X509Certificate::from_der(&pem.contents)
            .map_err(|e| Errors::parse("Invalid certificate structure", Some(Box::new(e))))?;

        Ok(Self { der: pem.contents })
    }

    pub fn der(&self) -> &[u8] {
        &self.der
    }

    pub fn thumbprint_sha256(&self) -> String {
        let hash = Sha256::digest(&self.der);
        URL_SAFE_NO_PAD.encode(hash)
    }

    pub fn check_validity(&self) -> Outcome<()> {
        let (_, cert) = X509Certificate::from_der(&self.der)
            .map_err(|e| Errors::security("Failed to re-parse certificate", Some(Box::new(e))))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Errors::security("System clock is before UNIX epoch", Some(Box::new(e))))?
            .as_secs() as i64;

        let not_before = cert.validity().not_before.timestamp();
        let not_after = cert.validity().not_after.timestamp();

        if now < not_before {
            return Err(Errors::security("Certificate is not yet valid", None));
        }
        if now > not_after {
            return Err(Errors::security("Certificate has expired", None));
        }
        Ok(())
    }

    pub fn public_key(&self) -> Outcome<PublicKey> {
        let (_, cert) = X509Certificate::from_der(&self.der)
            .map_err(|e| Errors::parse("Failed to re-parse certificate", Some(Box::new(e))))?;

        PublicKey::try_from_pkcs8_der(cert.public_key().raw)
    }
}

fn normalize_pem(cert: &str) -> String {
    let cert = cert.trim();
    if cert.starts_with("-----BEGIN CERTIFICATE-----") {
        return cert.to_string();
    }
    format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
        cert
    )
}
