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

use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::{Signer, Verifier};
use openssl::x509::X509;
use rand::Rng;
use rand_distr::Alphanumeric;
use sha2::{Digest, Sha256};

use crate::errors::{Errors, Outcome};

/// Tolerancia máxima en segundos entre el `created` de la firma y el tiempo actual.
const MAX_CLOCK_SKEW_SECS: u64 = 30;

/// Utilidad sin estado para construir y verificar headers HTTP Message Signatures
/// (RFC 9421) según el perfil GNAP (RFC 9635 sección 7.3.1).
pub struct HttpSig;

/// Resultado de la validación de un certificado.
#[derive(Debug, Clone, PartialEq)]
pub enum CertStatus {
    /// El certificado es válido y está dentro de su periodo de validez.
    Valid,
    /// El certificado ha expirado.
    Expired,
    /// El certificado aún no es válido (fecha de inicio en el futuro).
    NotYetValid,
    /// El certificado no pudo ser parseado o está malformado.
    Malformed
}

impl CertStatus {
    /// Devuelve true solo si el certificado está en estado Valid.
    pub fn is_valid(&self) -> bool { *self == CertStatus::Valid }
}

impl HttpSig {
    // =========================================================================
    // SIGNING — lado cliente
    // =========================================================================

    pub fn build(
        cert_pem: &str,
        private_key_pem: &str,
        method: &str,
        url: &str,
        body_bytes: &[u8],
        authorization: Option<&str>
    ) -> Outcome<HeaderMap> {
        let key_id = Self::compute_thumbprint(cert_pem)?;

        let created = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let nonce: String =
            rand::rng().sample_iter(&Alphanumeric).take(32).map(char::from).collect();

        let content_digest = Self::compute_content_digest(body_bytes);
        let content_length = body_bytes.len();

        let (signature_base, sig_params) = Self::build_signature_base(
            method,
            url,
            &content_digest,
            content_length,
            created,
            &key_id,
            &nonce,
            authorization
        );

        let signature = Self::sign_base(&signature_base, private_key_pem)?;

        let mut headers = HeaderMap::new();

        headers.insert(
            "content-digest",
            content_digest.parse().map_err(|e| {
                Errors::parse("Failed to parse content-digest header", Some(Box::new(e)))
            })?
        );
        headers.insert(
            "content-length",
            content_length.to_string().parse().map_err(|e| {
                Errors::parse("Failed to parse content-length header", Some(Box::new(e)))
            })?
        );
        headers.insert(
            "signature-input",
            format!("sig1={sig_params}").parse().map_err(|e| {
                Errors::parse("Failed to parse signature-input header", Some(Box::new(e)))
            })?
        );
        headers.insert(
            "signature",
            format!("sig1=:{signature}:").parse().map_err(|e| {
                Errors::parse("Failed to parse signature header", Some(Box::new(e)))
            })?
        );

        Ok(headers)
    }

    // =========================================================================
    // VERIFICATION — lado servidor
    // =========================================================================

    /// Verifica una request httpsig entrante.
    ///
    /// Devuelve:
    /// - `Ok(())`  — firma criptográficamente válida
    /// - `Err`     — firma inválida, request manipulada o malformada → 401
    ///
    /// Esta función solo verifica la firma. La decisión de si el cert
    /// es confiable o requiere aprobación manual se toma con `check_cert`.
    pub fn verify(
        headers: &HeaderMap,
        method: &str,
        url: &str,
        body_bytes: &[u8],
        cert_pem: &str
    ) -> Outcome<()> {
        let signature_input = Self::extract_header(headers, "signature-input")?;
        let signature_header = Self::extract_header(headers, "signature")?;
        let content_digest = Self::extract_header(headers, "content-digest")?;

        if !signature_input.contains("tag=\"gnap\"") {
            return Err(Errors::security(
                "Missing required tag=\"gnap\" in Signature-Input",
                None
            ));
        }

        let expected_digest = Self::compute_content_digest(body_bytes);
        if content_digest != expected_digest {
            return Err(Errors::security(
                "Content-Digest mismatch — body may have been tampered",
                None
            ));
        }

        let created =
            Self::extract_sig_param(&signature_input, "created")?.parse::<u64>().map_err(|_| {
                Errors::security("Invalid `created` timestamp in Signature-Input", None)
            })?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if now < created || now - created > MAX_CLOCK_SKEW_SECS {
            return Err(Errors::security(
                format!(
                    "Signature timestamp out of acceptable range \
                     (created={created}, now={now}, max_skew={MAX_CLOCK_SKEW_SECS}s)"
                ),
                None
            ));
        }

        let keyid_in_sig = Self::extract_sig_param(&signature_input, "keyid")?;
        let cert_thumbprint = Self::compute_thumbprint(cert_pem)?;

        if keyid_in_sig != cert_thumbprint {
            return Err(Errors::security(
                "keyid in Signature-Input does not match thumbprint of declared certificate",
                None
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

        let (reconstructed_base, _) = Self::build_signature_base(
            method,
            url,
            &content_digest,
            content_length,
            created,
            &keyid_in_sig,
            &nonce,
            authorization
        );

        Self::verify_signature(&reconstructed_base, &signature_bytes, cert_pem)
    }

    // =========================================================================
    // CERT VALIDATION — comprobación independiente del certificado
    // =========================================================================

    /// Comprueba la validez temporal de un certificado PEM.
    ///
    /// Devuelve un `CertStatus` que indica si el certificado es válido,
    /// ha expirado, aún no es válido, o está malformado.
    ///
    /// Esta función es independiente de httpsig — se puede usar en cualquier
    /// punto del flujo donde se necesite saber si un cert es temporalmente válido,
    /// por ejemplo antes de decidir si un participante necesita aprobación manual.
    ///
    /// Uso típico en el handler GNAP:
    /// ```rust
    /// HttpSig::verify(&headers, "POST", &url, &body, cert_pem)?; // firma válida o 401
    ///
    /// match HttpSig::check_cert(cert_pem) {
    ///     CertStatus::Valid      => process_automatic(grant_request).await,
    ///     CertStatus::Expired    => return StatusCode::UNAUTHORIZED,
    ///     CertStatus::NotYetValid => return StatusCode::UNAUTHORIZED,
    ///     CertStatus::Malformed  => return StatusCode::BAD_REQUEST,
    /// }
    /// ```
    pub fn check_cert(cert_pem: &str) -> Outcome<()> {
        let cert = X509::from_pem(cert_pem.as_bytes())
            .map_err(|e| Errors::security("Client certificate is malformed", Some(Box::new(e))))?;

        let now = openssl::asn1::Asn1Time::days_from_now(0)
            .expect("Failed to get current time as Asn1Time");

        let not_before = cert.not_before();
        let not_after = cert.not_after();

        match (not_before.compare(&now), now.compare(not_after)) {
            (
                Ok(std::cmp::Ordering::Less) | Ok(std::cmp::Ordering::Equal),
                Ok(std::cmp::Ordering::Less) | Ok(std::cmp::Ordering::Equal)
            ) => Ok(()),

            (Ok(std::cmp::Ordering::Greater), _) => {
                Err(Errors::security("Client certificate is not yet valid", None))
            }

            (_, Ok(std::cmp::Ordering::Greater)) => {
                Err(Errors::security("Client certificate has expired", None))
            }

            _ => Err(Errors::security("Client certificate is malformed", None))
        }
    }

    /// Devuelve el thumbprint SHA-256 de un certificado PEM como string base64url.
    /// Útil para identificar un cert sin almacenarlo completo.
    pub fn thumbprint(cert_pem: &str) -> Outcome<String> { Self::compute_thumbprint(cert_pem) }

    // =========================================================================
    // Internals
    // =========================================================================

    fn compute_thumbprint(cert_pem: &str) -> Outcome<String> {
        let cert = X509::from_pem(cert_pem.as_bytes())
            .map_err(|e| Errors::security("Failed to parse cert PEM", Some(Box::new(e))))?;

        let digest = cert.digest(MessageDigest::sha256()).map_err(|e| {
            Errors::security("Failed to compute cert thumbprint", Some(Box::new(e)))
        })?;

        Ok(URL_SAFE_NO_PAD.encode(digest))
    }

    fn compute_content_digest(body: &[u8]) -> String {
        let hash = Sha256::digest(body);
        format!("sha-256=:{}:", URL_SAFE_NO_PAD.encode(hash))
    }

    fn build_signature_base(
        method: &str,
        url: &str,
        content_digest: &str,
        content_length: usize,
        created: u64,
        key_id: &str,
        nonce: &str,
        authorization: Option<&str>
    ) -> (String, String) {
        let mut components: Vec<&str> =
            vec!["@method", "@target-uri", "content-digest", "content-length", "content-type"];

        if authorization.is_some() {
            components.push("authorization");
        }

        let component_list =
            components.iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(" ");

        let sig_params = format!(
            "({component_list})\
            ;created={created}\
            ;keyid=\"{key_id}\"\
            ;nonce=\"{nonce}\"\
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

    fn sign_base(signature_base: &str, private_key_pem: &str) -> Outcome<String> {
        let pkey = PKey::private_key_from_pem(private_key_pem.as_bytes())
            .map_err(|e| Errors::security("Failed to load private key", Some(Box::new(e))))?;

        let mut signer = Signer::new(MessageDigest::sha256(), &pkey)
            .map_err(|e| Errors::security("Failed to create signer", Some(Box::new(e))))?;

        signer
            .update(signature_base.as_bytes())
            .map_err(|e| Errors::security("Failed to update signer", Some(Box::new(e))))?;

        let signature = signer
            .sign_to_vec()
            .map_err(|e| Errors::security("Failed to produce signature", Some(Box::new(e))))?;

        Ok(URL_SAFE_NO_PAD.encode(signature))
    }

    fn verify_signature(
        signature_base: &str,
        signature_bytes: &[u8],
        cert_pem: &str
    ) -> Outcome<()> {
        let cert = X509::from_pem(cert_pem.as_bytes()).map_err(|e| {
            Errors::security("Failed to parse cert for verification", Some(Box::new(e)))
        })?;

        let public_key = cert.public_key().map_err(|e| {
            Errors::security("Failed to extract public key from cert", Some(Box::new(e)))
        })?;

        let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)
            .map_err(|e| Errors::security("Failed to create verifier", Some(Box::new(e))))?;

        verifier
            .update(signature_base.as_bytes())
            .map_err(|e| Errors::security("Failed to update verifier", Some(Box::new(e))))?;

        let valid = verifier
            .verify(signature_bytes)
            .map_err(|e| Errors::security("Failed to verify signature", Some(Box::new(e))))?;

        if !valid {
            return Err(Errors::security(
                "Signature verification failed — request rejected",
                None
            ));
        }

        Ok(())
    }

    fn extract_header(headers: &HeaderMap, name: &str) -> Outcome<String> {
        headers
            .get(name)
            .ok_or_else(|| Errors::security(format!("Missing required header: {name}"), None))?
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| {
                Errors::security(format!("Header {name} contains invalid UTF-8"), Some(Box::new(e)))
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
            None
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
