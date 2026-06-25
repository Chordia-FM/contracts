# contracts - Chordia

> The single source of truth for every API contract in the Chordia ecosystem.

[![CI](https://img.shields.io/badge/CI-passing-brightgreen)](#)
[![Release](https://img.shields.io/badge/release-0.1.0-blue)](#)
[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-orange)](./LICENSE)

## Overview

Every wire shape that crosses a Chordia process boundary is defined **once**, here, as a Rust
type. `backend`, `library`, and `dj` depend on this crate directly; `frontend` consumes the
**generated** TypeScript package (`@chordia/contracts`). One definition, no drift.

It also hosts the canonical [architecture doc](./docs/ARCHITECTURE.md) and
[README template](./docs/README_TEMPLATE.md). The contracts repo is the natural home for
shared, cross-repo documentation.

## Architecture

- **Stack:** Rust (pure types: `serde` + `uuid`), optional `ts-rs` for TS generation.
- **Responsibilities:** auth/capability tokens, users, social, libraries/shares, server
  directory, catalog + fingerprints, streaming/quality, scrobbles, insights, room messages.
- **Talks to:** nobody, it's a dependency, not a service.
- **MSRV:** `1.85` (current stable), consistent with the rest of the workspace.

## Conventions

- **Timestamps** = `i64` Unix epoch millis (`EpochMillis`) → TS `number`, no timezone ambiguity.
- **Ids** = `Uuid` (UUIDv7 where the producer mints them, so ids sort by time and double as
  idempotency keys, see `scrobble::ListeningEvent`).

## Generating TypeScript bindings

```bash
cargo test --features ts        # emits ./bindings/*.ts via ts-rs
# packaged + published as @chordia/contracts on tagged releases
```

The default build pulls **no** TS toolchain, keeping service builds lean.

## Development

```bash
cargo build          # verify types compile
cargo test           # contract round-trip tests
cargo clippy -D warnings
```

## Versioning

Semver on the wire surface; see [`docs/versioning.md`](./docs/versioning.md). Downstream repos
pin a compatible range; a contract-drift CI job opens auto-PRs when a new version lands.

## License

AGPL-3.0, see [LICENSE](./LICENSE).
