//! Server-directory contracts. The Hub is a rendezvous: self-hosted libraries advertise where
//! they are reachable and what TLS fingerprint to pin, so clients can connect directly + safely.

use serde::{Deserialize, Serialize};

use crate::auth::{CapabilityAction, ResourceRef};
use crate::{EpochMillis, Uuid};

/// A library server's current reachability record in the Hub directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ServerEndpoint {
    pub server_id: Uuid,
    pub owner_id: Uuid,
    /// Reachable base URL, e.g. `https://music.example.com:8443` or `https://203.0.113.7:8443`.
    pub endpoint: String,
    /// SHA-256 of the server's TLS leaf certificate (hex). Clients **pin** this, making
    /// self-signed certs safe against MITM.
    pub tls_fingerprint: String,
    pub online: bool,
    pub last_heartbeat: EpochMillis,
}

/// Heartbeat a library posts to the Hub on a fixed interval to stay "online" and keep its
/// endpoint/fingerprint current (handles dynamic IPs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HeartbeatRequest {
    pub server_id: Uuid,
    pub endpoint: String,
    pub tls_fingerprint: String,
}

/// Hub response to a heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HeartbeatResponse {
    pub ok: bool,
    /// Seconds until the next expected heartbeat (the reaper marks stale servers offline).
    pub next_interval_secs: u32,
}

/// Result of resolving a server before initiating a direct stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolvedServer {
    pub endpoint: ServerEndpoint,
    /// True if the Hub also issued a capability token alongside this resolution.
    pub authorized: bool,
}

/// Request a capability (or relay) token for a resource on a friend's / DJ's library
/// (`POST /v1/directory/grant`). The Hub checks friendship + share permissions before minting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GrantRequest {
    pub library_id: Uuid,
    pub resource: ResourceRef,
    pub action: CapabilityAction,
    /// Room context, required for relay grants.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<Uuid>,
}

/// A minted capability token + the resolved server to present it to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GrantResponse {
    pub token: String,
    pub server: ServerEndpoint,
    pub expires_at: EpochMillis,
}
