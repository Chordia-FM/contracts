//! Scrobble (listening-event) contracts - the input to the centralized "Wrapped" engine.

use serde::{Deserialize, Serialize};

use crate::{catalog::TrackFingerprint, EpochMillis, Uuid};

/// Where the bytes the user heard came from. Stored so insights can distinguish personal
/// listening from room/relay listening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum PlaybackSource {
    /// Played from a file cached on the device itself.
    Local,
    /// Streamed from the user's own library.
    OwnLibrary,
    /// Relayed through the user's library from a DJ's library.
    Relay,
    /// Streamed directly from a friend's shared library.
    Friend,
}

/// The originating client type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ClientType {
    Web,
    Desktop,
    Mobile,
}

/// A single listening event. Created client-side with a UUIDv7 `event_id` so it doubles as the
/// **idempotency key** - replays/reinstalls never double-count.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ListeningEvent {
    /// UUIDv7 - time-sortable + idempotency key.
    pub event_id: Uuid,
    /// Identity of what was played (resolved to the canonical catalog server-side).
    pub fingerprint: TrackFingerprint,
    /// Track title exactly as reported by the client. Optional for compatibility with older clients;
    /// resolved events can fall back to the canonical catalog title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Artist display string exactly as reported by the client. This is deliberately distinct from
    /// `fingerprint.artist_norm`, which is only a fuzzy-matching key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    /// When playback started (epoch millis, client clock).
    pub started_at: EpochMillis,
    /// Milliseconds actually played.
    pub ms_played: u32,
    /// Track duration for threshold/skew calculations.
    pub duration_ms: u32,
    pub source: PlaybackSource,
    pub client_type: ClientType,
    /// Library streamed from, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_id: Option<Uuid>,
    /// Room the track was heard in, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<Uuid>,
    /// Playlist the track was played from, when the play originated in a playlist context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playlist_id: Option<Uuid>,
}

/// An owner's correction to one of their own listening events
/// (`PATCH /v1/scrobbles/{event_id}`, a Super-Sonic capability).
///
/// At least one field must be present. Re-attributing (`track_id`) rewrites the event's canonical
/// ids AND its denormalized display text together; moving it (`started_at`) may relocate the row
/// to a different monthly partition. Either way the rollups are repaired by exact recompute.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScrobbleEdit {
    /// Re-attribute the play to this track (must be in a library the caller can access).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<Uuid>,
    /// Move the play to this start time (epoch millis, bounded to [2000-01-01, now + 5 min]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<EpochMillis>,
}

/// A batch flush of buffered events (`POST /v1/scrobbles:batch`). Clients buffer offline and
/// flush on reconnect.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScrobbleBatch {
    pub events: Vec<ListeningEvent>,
}

/// Ingestion result. `accepted` + `duplicates` should equal the batch size.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ScrobbleBatchResponse {
    pub accepted: u32,
    /// Events deduped by `event_id` (already ingested).
    pub duplicates: u32,
    /// Event ids rejected as malformed, if any.
    #[serde(default)]
    pub rejected: Vec<Uuid>,
}
