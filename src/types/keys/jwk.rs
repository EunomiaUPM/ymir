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


use crate::errors::{BadFormat, Errors, Outcome};
use crate::utils::{decode_url_safe_no_pad, encode_url_safe_no_pad};
use ed25519_dalek::VerifyingKey as Ed25519VerifyingKey;
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, RsaPublicKey};
use serde_json::{Value, json};

pub(super) fn rsa_public_key_from_jwk(jwk: &Value) -> Outcome<RsaPublicKey> {
    let n_b64 = jwk
        .get("n")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Errors::format(BadFormat::Received, "RSA JWK missing 'n'", None))?;
    let e_b64 = jwk
        .get("e")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Errors::format(BadFormat::Received, "RSA JWK missing 'e'", None))?;

    let n_bytes = decode_url_safe_no_pad(n_b64)?;
    let e_bytes = decode_url_safe_no_pad(e_b64)?;

    let n = BigUint::from_bytes_be(&n_bytes);
    let e = BigUint::from_bytes_be(&e_bytes);

    RsaPublicKey::new(n, e).map_err(|err| {
        Errors::format(
            BadFormat::Received,
            "Invalid RSA public key components",
            Some(Box::new(err)),
        )
    })
}

pub(super) fn ed25519_public_key_from_jwk(jwk: &Value) -> Outcome<Ed25519VerifyingKey> {
    let x_b64 = jwk
        .get("x")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Errors::format(BadFormat::Received, "OKP JWK missing 'x'", None))?;

    let x_bytes = decode_url_safe_no_pad(x_b64)?;

    let arr: [u8; 32] = x_bytes.as_slice().try_into().map_err(|_| {
        Errors::format(
            BadFormat::Received,
            format!("Ed25519 public key must be 32 bytes, got {}", x_bytes.len()),
            None,
        )
    })?;

    Ed25519VerifyingKey::from_bytes(&arr).map_err(|err| {
        Errors::format(
            BadFormat::Received,
            "Invalid Ed25519 public key bytes",
            Some(Box::new(err)),
        )
    })
}

pub(super) fn rsa_public_jwk(pk: &RsaPublicKey) -> Value {
    json!({
        "kty": "RSA",
        "n": encode_url_safe_no_pad(pk.n().to_bytes_be()),
        "e": encode_url_safe_no_pad(pk.e().to_bytes_be()),
    })
}

pub(super) fn ed25519_public_jwk(vk: &Ed25519VerifyingKey) -> Value {
    json!({
        "kty": "OKP",
        "crv": "Ed25519",
        "x": encode_url_safe_no_pad(vk.to_bytes()),
    })
}
