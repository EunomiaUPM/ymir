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

use crate::services::issuer::IssuerTrait;
use crate::services::vault::VaultService;
use crate::services::verifier::VerifierTrait;
use crate::services::wallet::WalletTrait;
use std::sync::Arc;

/// Capability provider for the Decentralized Identity Wallet core service.
///
/// Implemented by high-level business modules to pass down wallet operations
/// (such as DID management and verifiable presentation generation) into HTTP request states.
pub trait HasWallet {
    /// Returns a reference-counted pointer to the active Wallet service trait object.
    fn wallet(&self) -> Arc<dyn WalletTrait>;
}

/// Capability provider for the OpenID4VP Presentation Verification service.
///
/// Used by verification-enabled endpoints to validate presentation exchanges
/// without directly binding the router to concrete architecture engines.
pub trait HasVerifier {
    /// Returns a reference-counted pointer to the active Verifier service trait object.
    fn verifier(&self) -> Arc<dyn VerifierTrait>;
}

/// Capability provider for the OpenID4VCI Credential Issuance service.
///
/// Binds issuance state logic to business orchestration modules, exposing cryptographic
/// signing capabilities for outbound credentials.
pub trait HasIssuer {
    /// Returns a reference-counted pointer to the active Issuer service trait object.
    fn issuer(&self) -> Arc<dyn IssuerTrait>;
}

/// Capability provider for the Secure Cryptographic Secret Vault engine.
///
/// Yields direct access to the `VaultService` enum dispatcher. This circumvents
/// unnecessary trait vtable lookups, delivering highly efficient access to
/// Sea-ORM connection strings and raw asymmetric PEM keys.
pub trait HasVault {
    /// Returns a reference-counted pointer to the operational Vault dispatcher.
    fn vault(&self) -> Arc<VaultService>;
}