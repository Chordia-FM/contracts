//! Server-directory contracts. The Hub is a rendezvous: self-hosted libraries advertise where
//! they are reachable and what TLS fingerprint to pin, so clients can connect directly + safely.

use serde::{Deserialize, Serialize};

use crate::auth::{CapabilityAction, ResourceRef};
use crate::library::PermissionLevel;
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

/// Who owns a library server, as the Hub tells the server itself (`GET /v1/directory/me`).
/// The library's Discord bots treat the owner as an implicit bot owner, through the Discord
/// account the owner linked to Chordia, if any.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ServerOwner {
    pub server_id: Uuid,
    pub owner_id: Uuid,
    pub handle: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discord_id: Option<String>,
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
    /// What the token covers. Omit for the whole library, which is the ordinary streaming case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<ResourceRef>,
    /// What the token permits. Omit for [`CapabilityAction::StreamRead`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<CapabilityAction>,
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
    /// The level this grant carries, signed into the token as well. Clients read it to know
    /// whether keeping a copy is allowed at all: `Read` is stream-only, and the library refuses a
    /// download-shaped request made with such a token.
    pub permission_level: PermissionLevel,
}

/// A server paired to the caller's account, as the owner sees it
/// (`GET /v1/servers`). The API key itself is never included — it is returned exactly once, by
/// pairing, and afterwards only ever replaced by a rotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PairedServer {
    pub server_id: Uuid,
    /// Last endpoint the server advertised, if it has ever heartbeat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// The pinned TLS leaf fingerprint the server last advertised (hex SHA-256).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_fingerprint: Option<String>,
    pub online: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat: Option<EpochMillis>,
    /// Names of the logical libraries this server backs. Unpairing the server removes them too,
    /// so the owner can see what a revoke would take with it.
    pub libraries: Vec<String>,
}

/// A freshly rotated server API key (`POST /v1/servers/{server_id}/rotate-key`). The previous key
/// stops authenticating immediately, so the library has to be re-paired or reconfigured with this
/// one.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RotatedServerKey {
    pub server_id: Uuid,
    pub server_api_key: String,
}
