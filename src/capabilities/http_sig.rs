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

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

use rand::Rng;
use rand::distributions::Alphanumeric;
use sha2::{Digest, Sha256};

use crate::errors::{Errors, Outcome};
use crate::types::keys::{Alg, KeySource, PrivateKey};

const MAX_CLOCK_SKEW_SECS: u64 = 30;

pub struct HttpSig;

impl HttpSig {
    // =========================================================================
    // SIGNING — client side
    // =========================================================================

    pub fn build(
        key_source: &KeySource,
        priv_key: &PrivateKey,
        alg: Option<Alg>,
        method: &str,
        url: &str,
        body_bytes: &[u8],
        authorization: Option<&str>,
    ) -> Outcome<HeaderMap> {
        let alg = alg.unwrap_or(priv_key.alg());
        let key_id = key_source.thumbprint();
        let created = unix_now();
        let nonce = random_nonce_32();
        let content_digest = digest(body_bytes);
        let content_length = body_bytes.len();

        let (signature_base, sig_params) = Self::build_signature_base(
            method,
            url,
            &content_digest,
            content_length,
            created,
            &key_id,
            &nonce,
            &alg,
            authorization,
        );

        let signature_bytes = priv_key.sign_bytes(signature_base.as_bytes(), alg)?;
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature_bytes);

        let mut headers = HeaderMap::new();
        headers.insert(
            "content-digest",
            content_digest.parse().map_err(|e| {
                Errors::parse("Failed to parse content-digest header", Some(Box::new(e)))
            })?,
        );
        headers.insert(
            "content-length",
            content_length.to_string().parse().map_err(|e| {
                Errors::parse("Failed to parse content-length header", Some(Box::new(e)))
            })?,
        );
        headers.insert(
            "signature-input",
            format!("sig1={sig_params}").parse().map_err(|e| {
                Errors::parse("Failed to parse signature-input header", Some(Box::new(e)))
            })?,
        );
        headers.insert(
            "signature",
            format!("sig1=:{signature_b64}:").parse().map_err(|e| {
                Errors::parse("Failed to parse signature header", Some(Box::new(e)))
            })?,
        );

        Ok(headers)
    }

    // =========================================================================
    // VERIFICATION — server side
    // =========================================================================

    pub fn verify(
        headers: &HeaderMap,
        key_source: &KeySource,
        method: &str,
        url: &str,
        body_bytes: &[u8],
    ) -> Outcome<()> {
        let signature_input = Self::extract_header(headers, "signature-input")?;
        let signature_header = Self::extract_header(headers, "signature")?;
        let content_digest = Self::extract_header(headers, "content-digest")?;

        key_source.check_validity()?;

        if !signature_input.contains("tag=\"gnap\"") {
            return Err(Errors::security(
                "Missing required tag=\"gnap\" in Signature-Input",
                None,
            ));
        }

        let expected_digest = digest(body_bytes);
        if content_digest != expected_digest {
            return Err(Errors::security(
                "Content-Digest mismatch — body may have been tampered",
                None,
            ));
        }

        let created = Self::extract_sig_param(&signature_input, "created")?
            .parse::<u64>()
            .map_err(|_| {
                Errors::security("Invalid `created` timestamp in Signature-Input", None)
            })?;

        check_clock_skew(created)?;

        let keyid_in_sig = Self::extract_sig_param(&signature_input, "keyid")?;
        let cert_thumbprint = key_source.thumbprint();

        if keyid_in_sig != cert_thumbprint {
            return Err(Errors::security(
                "keyid in Signature-Input does not match thumbprint of declared certificate",
                None,
            ));
        }

        let signature_value = Self::extract_sig_value(&signature_header)?;
        let signature_bytes = URL_SAFE_NO_PAD
            .decode(signature_value)
            .map_err(|e| Errors::security("Failed to decode signature value", Some(Box::new(e))))?;

        let authorization = if signature_input.contains("\"authorization\"") {
            headers.get("authorization").and_then(|v| v.to_str().ok())
        } else {
            None
        };

        let content_length = body_bytes.len();
        let nonce = Self::extract_sig_param(&signature_input, "nonce").unwrap_or_default();

        let alg_str = Self::extract_sig_param(&signature_input, "alg")?;
        let Ok(alg) = Alg::from_str(&alg_str);

        let (reconstructed_base, _) = Self::build_signature_base(
            method,
            url,
            &content_digest,
            content_length,
            created,
            &keyid_in_sig,
            &nonce,
            &alg,
            authorization,
        );

        key_source.verify_bytes(reconstructed_base.as_bytes(), &signature_bytes, &alg)
    }

    // =========================================================================
    // Internals
    // =========================================================================

    fn build_signature_base(
        method: &str,
        url: &str,
        content_digest: &str,
        content_length: usize,
        created: u64,
        key_id: &str,
        nonce: &str,
        alg: &Alg,
        authorization: Option<&str>,
    ) -> (String, String) {
        let mut components: Vec<&str> = vec![
            "@method",
            "@target-uri",
            "content-digest",
            "content-length",
            "content-type",
        ];

        if authorization.is_some() {
            components.push("authorization");
        }

        let component_list = components
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(" ");

        let sig_params = format!(
            "({component_list})\
            ;created={created}\
            ;keyid=\"{key_id}\"\
            ;nonce=\"{nonce}\"\
            ;alg=\"{alg}\"\
            ;tag=\"gnap\""
        );

        let mut lines = vec![
            format!("\"@method\": {}", method.to_uppercase()),
            format!("\"@target-uri\": {url}"),
            format!("\"content-digest\": {content_digest}"),
            format!("\"content-length\": {content_length}"),
            "\"content-type\": application/json".to_string(),
        ];

        if let Some(auth) = authorization {
            lines.push(format!("\"authorization\": {auth}"));
        }

        lines.push(format!("\"@signature-params\": {sig_params}"));

        (lines.join("\n"), sig_params)
    }

    fn extract_header(headers: &HeaderMap, name: &str) -> Outcome<String> {
        headers
            .get(name)
            .ok_or_else(|| Errors::security(format!("Missing required header: {name}"), None))?
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| {
                Errors::security(
                    format!("Header {name} contains invalid UTF-8"),
                    Some(Box::new(e)),
                )
            })
    }

    fn extract_sig_param(signature_input: &str, param: &str) -> Outcome<String> {
        let quoted_pattern = format!("{param}=\"");
        if let Some(start) = signature_input.find(&quoted_pattern) {
            let rest = &signature_input[start + quoted_pattern.len()..];
            if let Some(end) = rest.find('"') {
                return Ok(rest[..end].to_string());
            }
        }

        let plain_pattern = format!("{param}=");
        if let Some(start) = signature_input.find(&plain_pattern) {
            let rest = &signature_input[start + plain_pattern.len()..];
            let end = rest
                .find(|c: char| c == ';' || c == ',' || c == ')' || c == ' ')
                .unwrap_or(rest.len());
            return Ok(rest[..end].to_string());
        }

        Err(Errors::security(
            format!("Parameter `{param}` not found in Signature-Input"),
            None,
        ))
    }

    fn extract_sig_value(signature_header: &str) -> Outcome<&str> {
        let start = signature_header.find("=:").ok_or_else(|| {
            Errors::security("Malformed Signature header — expected sig1=:<value>:", None)
        })?;

        let rest = &signature_header[start + 2..];

        let end = rest.rfind(':').ok_or_else(|| {
            Errors::security("Malformed Signature header — missing closing ':'", None)
        })?;

        Ok(&rest[..end])
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn random_nonce_32() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

fn digest(body: &[u8]) -> String {
    let hash = Sha256::digest(body);
    format!("sha-256=:{}:", URL_SAFE_NO_PAD.encode(hash))
}

fn check_clock_skew(created: u64) -> Outcome<()> {
    let now = unix_now();
    if now < created || now - created > MAX_CLOCK_SKEW_SECS {
        return Err(Errors::security(
            format!(
                "Signature timestamp out of acceptable range \
                 (created={created}, now={now}, max_skew={MAX_CLOCK_SKEW_SECS}s)"
            ),
            None,
        ));
    }
    Ok(())
}
