# Ymir

> Core crate of the EUNOMIA Self-Sovereign Identity ecosystem.

Ymir is the shared foundation consumed by the SSI agents in the EUNOMIA stack
(`ds-agent`, `heimdall`, and any other participant). It provides the data
model, repository traits, GNAP / OID4VCI / OID4VP type definitions, wallet
abstractions, and cryptographic capabilities that every node needs in order to
speak the protocol consistently.

Ymir does **not** bind to any specific HTTP framework, database backend, or
wallet implementation by default — it exposes traits and concrete defaults
(`PostgresRepo`, `FafnirWallet`, etc.) that downstream crates pick and wire
together.

---

## At a glance

| Concern                | Where to look                                                |
|------------------------|--------------------------------------------------------------|
| Persistent data model  | `src/data/entities/{sent, received, shared, wallet}`         |
| Database migrations    | `src/data/migrations`                                        |
| Service traits         | `src/services/{repo, wallet, issuer, verifier, ...}`         |
| Module-level traits    | `src/modules`                                                |
| Protocol type system   | `src/types/{gnap, jwt, vcs, vps, dids, keys, ...}`           |
| Cryptographic primitives | `src/capabilities` (DID resolution, HTTP signatures, signers, verifiers) |
| Error model            | `src/errors`                                                 |
| HTTP helpers           | `src/http`                                                   |
| Declarative macros     | `src/macros` (`impl_serde_via_str!`, `impl_seaorm_via_str!`) |

---

## Architecture in three layers

Ymir follows a three-layer separation that every consumer (ds-agent, heimdall)
inherits:

1. **Data** — SeaORM entities, migrations, and repository traits.
   The schema is split into three namespaces:
   - `sent` — grants the local agent initiated (`peer_connector`,
     `vc_requester` flows).
   - `received` — grants the local agent is processing on behalf of a peer
     (`gatekeeper`, `approver` flows).
   - `shared` — entities that exist regardless of direction
     (`participant`, `resource_req`, `issuance`).
   - `wallet` — DIDs, keys and stored verifiable credentials.

2. **Services** — primitive operations behind trait boundaries. They expose
   capabilities a downstream module orchestrates: persistence (`repo`),
   issuance, verification, wallet operations, vault access, HTTP client.
   Concrete implementations live in submodules
   (e.g. `services/wallet/fafnir`, `services/repo/postgres`).

3. **Modules** — high-level orchestrators that compose services into protocol
   flows. Ymir ships base modules (`wallet::WalletModule`); downstream crates
   add the protocol-specific ones (`PeerConnectorModule`,
   `VcRequesterModule`, `GatekeeperModule`, …).

The `Has*` family of traits (`HasRepo`, `HasWallet`, `HasVerifier`, …) acts as
the dependency-injection contract between modules and the services they need.

---

## Type system: GNAP and SSI primitives

`src/types/gnap` is a typed model of the
[GNAP](https://datatracker.ietf.org/doc/draft-ietf-gnap-core-protocol/) protocol
used across the ecosystem:

- **`grant_request`** — full request shape: `client`, `access[]`,
  `interact`, `subject`.
- **`grant_response`** — `Approved` / `Pending` / `Processing` / `Error`,
  including `interact_response` and `continuation`.
- **`access_token`** — access and continue tokens, lifetimes, flags.
- **`callback`** — finish-interaction payloads (`Approved` / `Rejected`).
- **`status`** — `GrantStatus`, `VerificationStatus`, `InteractionFinishResponse`.

Around GNAP, ymir exposes the other SSI building blocks:

- `types/dids` — DID document, verification methods, services. Supports
  `did:web` and `did:jwk`.
- `types/keys` — JWK, key material, key proofs (for GNAP client binding).
- `types/jwt` — JWS / JWT header and payload helpers.
- `types/vcs`, `types/vps` — Verifiable Credentials and Presentations
  (W3C VC Data Model + JWT envelope).
- `types/issuance` — OID4VCI issuance plan (credential offer, build context).
- `types/verification` — OID4VP verification plan (presentation definition).
- `types/wallet` — wallet-facing payloads (`OidcUri`, etc.).

Enums that are closed (e.g. `GrantStatus`) derive `DeriveActiveEnum`.
Enums that must round-trip an unknown value through the database derive
serde via the `impl_seaorm_via_str!` macro defined in `src/macros`.

---

## The `Plan` pattern

Every entity in `data/entities` exposes a `Plan` type and the corresponding
`Model`. Modules build `Plan`s (intent — not yet persisted) and pass them to
a repository to obtain a `Model` (intent + DB identity + timestamps).

This makes building, testing, and persisting orthogonal concerns:

```rust
let grant_plan = self.peer_connector().build_grant_plan(payload);
let grant: grant::Model = self.repo().sent_grant().create(grant_plan).await?;
```

---

## Wallet abstraction

`services/wallet/wallet_trait.rs` defines `WalletTrait`: a single interface
for everything an agent needs to do with credentials and DIDs (issue, present,
verify, sign, rotate keys, resolve `did:web`, etc.).

Two implementations live in-tree:

- **`fafnir`** — first-party, in-process wallet built on the cryptographic
  capabilities of ymir. The default.
- **`walt_id`** — legacy HTTP adapter that talks to an external WaltID
  instance. Kept around for compatibility while migration completes.

Selection happens at composition time (the binary picks one `Box<dyn WalletTrait>`).

---

## Cryptographic capabilities

`src/capabilities/` exposes small, focused crypto modules used everywhere:

- `signer.rs`, `verifier.rs` — JWS signing/verification (Ed25519, RSA).
- `did.rs` — DID resolution and verification-method extraction.
- `http_sig.rs` — HTTP Message Signatures for GNAP client binding.
- `digest_sri.rs` — SHA-256 content digest helpers (`sha-256=:...:`).
- `kid.rs` — key-id helpers for JWK and DID URLs.

All crypto is pure Rust (RustCrypto family — no OpenSSL system dependency).
TLS uses `rustls`. JSON canonicalisation uses
[`json-canon`](https://crates.io/crates/json-canon) (RFC 8785).

---

## Database

- **ORM**: SeaORM 1.1.x with `sqlx-postgres` and `sqlx-sqlite`.
- **Migrations**: `src/data/migrations` is a regular SeaORM migration crate;
  consumers run them from their own binary at startup.
- **Schema layout**:
  - `sent_grants`, `sent_interactions`, `sent_verifications`
  - `recv_grants`, `recv_interactions`, `recv_verifications`
  - `resources_reqs`, `issuances`, `participants`
  - `wallet_dids`, `wallet_keys`, `wallet_vcs`
- **JSONB** is used for fields that carry structured arrays
  (`vc_type_config`, `actions`, `flags`, `build_ctx`); `text[]` is reserved
  for plain `Vec<String>` columns.

---

## Errors

`src/errors` defines `Errors`, `Outcome<T>` and `AppResult<T>` — the universal
error currency in the ecosystem. `Errors::log()` produces structured `tracing`
events. The helper `errors_to_error_code` converts internal errors into the
GNAP-spec `ErrorCode` set returned to peers.

---

## Using ymir from a downstream crate

Pin ymir to a published git tag in the consumer's `Cargo.toml`:

```toml
[dependencies]
ymir = { git = "https://github.com/EunomiaUPM/ymir.git", tag = "v0.8.0" }
```

Replace the `tag` with whichever release you target — see
[the tags on GitHub](https://github.com/EunomiaUPM/ymir/tags) for the
available versions. Use `rev = "<sha>"` instead of `tag` when you need to
pin to an unreleased commit, and `branch = "main"` only for short-lived
experimentation.

For local hacking against an in-tree checkout, override with a path in
`Cargo.toml` (or in `.cargo/config.toml` via `[patch.crates-io]` /
`[patch."https://github.com/EunomiaUPM/ymir.git"]`):

```toml
[patch."https://github.com/EunomiaUPM/ymir.git"]
ymir = { path = "../ymir" }
```

A minimal agent wires:

1. A concrete `Repo` (typically `PostgresRepo::new(db)`).
2. A concrete `Wallet` (`FafnirWallet::new(...)`).
3. The module traits it cares about (e.g. `PeerConnectorModule`,
   `GatekeeperModule`) implemented on a service struct that satisfies the
   required `Has*` bounds.
4. An HTTP router exposing the routes (each agent owns its own router crate).

`ds-agent` and `heimdall` in this workspace are the canonical examples.

---

## Project context

Ymir is part of the **EUNOMIA** project, developed at the
**Universidad Politécnica de Madrid (UPM)**. It is the shared kernel for a
dataspace of SSI agents that exchange data and credentials using GNAP for
authorisation and OID4VCI / OID4VP for credential issuance and presentation.

Sibling crates in the workspace:

- **`ds-agent`** — dataspace participant agent.
- **`heimdall`** — issuing authority / verifier agent.
- **`fafnir-wallet`** — embedded wallet implementation used by ymir.

---

## License

Copyright © 2026 Universidad Politécnica de Madrid.
Released under the **GNU General Public License v3.0** — see source headers
for the full notice.
