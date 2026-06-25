//! Live listening-room contracts (the `dj` service). Foundational for the MVP - defined now so
//! clients and services share the wire format, built out post-MVP.

use serde::{Deserialize, Serialize};

use crate::{catalog::TrackFingerprint, user::PublicUser, EpochMillis, Uuid};

/// The "now playing" descriptor broadcast to a room. Carries the full layered fingerprint so each
/// listener can run own-copy resolution locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NowPlaying {
    pub fingerprint: TrackFingerprint,
    /// Display metadata (so clients without an own-copy still render the track).
    pub title: String,
    pub artist: String,
    /// The DJ whose library is the relay source of truth.
    pub dj_user_id: Uuid,
    pub dj_server_id: Uuid,
    /// Authoritative playhead at `as_of` (room clock).
    pub position_ms: u32,
    pub as_of: EpochMillis,
    pub paused: bool,
}

/// A queued track waiting to be played.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct QueueItem {
    pub queue_id: Uuid,
    pub fingerprint: TrackFingerprint,
    pub title: String,
    pub artist: String,
    pub added_by: Uuid,
    pub votes: i32,
}

/// Full room snapshot (sent on join, then kept current via deltas).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RoomState {
    pub room_id: Uuid,
    pub title: String,
    pub current_dj: Option<Uuid>,
    pub listeners: Vec<PublicUser>,
    pub now_playing: Option<NowPlaying>,
    pub queue: Vec<QueueItem>,
}

/// Messages a client sends over the room WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ClientMessage {
    Join {
        room_id: Uuid,
    },
    Leave,
    /// DJ transport controls (ignored unless the sender is the current DJ).
    Play,
    Pause,
    Seek {
        position_ms: u32,
    },
    Next,
    Enqueue {
        fingerprint: TrackFingerprint,
    },
    Vote {
        queue_id: Uuid,
        up: bool,
    },
    Chat {
        body: String,
    },
    /// Latency probe for room-clock synchronization (echoed back).
    Ping {
        client_time: EpochMillis,
    },
}

/// Messages the `dj` service broadcasts to room clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ServerMessage {
    /// Full state on join.
    Snapshot {
        state: RoomState,
    },
    NowPlaying {
        now_playing: NowPlaying,
    },
    QueueUpdated {
        queue: Vec<QueueItem>,
    },
    ListenerJoined {
        user: PublicUser,
    },
    ListenerLeft {
        user_id: Uuid,
    },
    DjChanged {
        dj_user_id: Option<Uuid>,
    },
    Chat {
        from: Uuid,
        body: String,
        at: EpochMillis,
    },
    /// Clock-sync echo: `client_time` round-trips so the client can compute offset/RTT.
    Pong {
        client_time: EpochMillis,
        server_time: EpochMillis,
    },
    Error {
        message: String,
    },
}
