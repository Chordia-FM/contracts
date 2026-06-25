//! Streaming contracts: quality tiers and the relay request shape.
//!
//! Bit-perfect delivery itself is just HTTP Range over the original bytes - there's no custom
//! envelope for the `Original` tier. These types describe tier selection and relay setup.

use serde::{Deserialize, Serialize};

use crate::Uuid;

/// Listener-selectable quality tier. `Original` is the default and is byte-for-byte lossless;
/// lower tiers are produced by on-the-fly transcode and may be auto-selected when
/// `auto_downgrade` is enabled and bandwidth can't sustain the current tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum QualityProfile {
    /// Original file bytes, unaltered. Lossless / spatial passthrough.
    #[default]
    Original,
    /// ~256 kbps.
    High,
    /// ~128 kbps.
    Normal,
    /// ~96 kbps Opus, for metered connections.
    DataSaver,
}

/// Concrete encoding parameters a non-`Original` tier maps to (server transcode target).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TranscodeTarget {
    pub codec: String,
    pub bitrate_kbps: u16,
}

/// Query params for the stream endpoint (`GET /v1/stream/{track_id}`). The `Range` header drives
/// byte positioning; this only selects the tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct StreamQuery {
    #[serde(default)]
    pub profile: QualityProfile,
}

/// Per-network-class playback preference, persisted client-side and synced to the Hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QualityPreferences {
    pub wifi: QualityProfile,
    pub cellular: QualityProfile,
    /// Step the tier down automatically when buffer health degrades (and back up on recovery).
    pub auto_downgrade: bool,
}

/// Request from a client to its **own** library to relay a track it does not own from the DJ's
/// library (the "Hybrid Relay" fallback).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RelayRequest {
    /// Reachable base URL of the DJ's library (from the Hub directory).
    pub dj_endpoint: String,
    /// TLS fingerprint to pin on the relay pull.
    pub dj_tls_fingerprint: String,
    /// Track to fetch on the DJ's side.
    pub track_id: Uuid,
    /// Hub-minted relay capability token (audience = DJ library).
    pub relay_token: String,
    /// Room context.
    pub room_id: Uuid,
    #[serde(default)]
    pub profile: QualityProfile,
}
