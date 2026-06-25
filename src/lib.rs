//! # chordia-contracts
//!
//! The single source of truth for every API contract in the Chordia ecosystem.
//!
//! These Rust types are hand-authored here and consumed by `backend`, `library`, and `dj`.
//! TypeScript bindings for `frontend` / `@chordia/contracts` are generated from these same
//! types (see below), so there is exactly one definition of every wire shape.
//!
//! ## Conventions
//! - Timestamps are `i64` Unix epoch milliseconds ([`EpochMillis`]). This keeps the contract
//!   trivially portable to TypeScript (`number`) and avoids timezone ambiguity on the wire.
//! - Identifiers are [`Uuid`] (UUIDv7 where the producer controls creation, so IDs are
//!   time-sortable and double as idempotency keys, see [`scrobble::ListeningEvent`]).
//!
//! ## Generating TypeScript bindings
//! ```bash
//! cargo test --features ts      # emits ./bindings/*.ts via ts-rs
//! ```
//! The default build never pulls the TS toolchain, keeping service builds lean.

pub use uuid::Uuid;

/// Unix epoch milliseconds. Serialized as a JSON `number` / TS `number`.
pub type EpochMillis = i64;

/// Semver of the contract surface. Bumped on any breaking wire change; downstream repos pin a
/// compatible range. See `docs/versioning.md`.
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod artists;
pub mod auth;
pub mod catalog;
pub mod directory;
pub mod discovery;
pub mod insights;
pub mod library;
pub mod lyrics;
pub mod overrides;
pub mod pins;
pub mod room;
pub mod scrobble;
pub mod smart;
pub mod social;
pub mod streaming;
pub mod user;
