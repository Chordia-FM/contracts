//! Library & selective-sharing contracts. A user's self-hosted server may host several logical
//! **libraries** (named collections), each shared independently.

use serde::{Deserialize, Serialize};

use crate::{user::PublicUser, EpochMillis, Uuid};

/// What a grantee may do with a shared library. Read-only for the MVP; `Download` is reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum PermissionLevel {
    /// Stream only.
    Read,
    /// Stream + persist an own-copy (reserved, post-MVP).
    Download,
}

/// Summary of a logical library as registered with the Hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibrarySummary {
    pub id: Uuid,
    pub owner_id: Uuid,
    /// Display name, e.g. "Hi-Fi Archive", "Family Collection".
    pub name: String,
    /// The self-hosted server (`server_id`) that physically hosts this library.
    pub server_id: Uuid,
    /// Optional Phosphor icon name chosen by the owner; None → default icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub track_count: u32,
    pub created_at: EpochMillis,
}

/// A share grant: which user may access which library, at what level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibraryShare {
    pub library_id: Uuid,
    pub grantee: PublicUser,
    pub permission_level: PermissionLevel,
    /// Whether the grantee may queue acquisition (download) requests INTO this library.
    #[serde(default)]
    pub can_request: bool,
    pub created_at: EpochMillis,
}

/// Create or update a share (Hub-side, requires an accepted friendship).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ShareRequest {
    pub library_id: Uuid,
    pub grantee_id: Uuid,
    pub permission_level: PermissionLevel,
    /// Allow the grantee to queue acquisition requests into this library (default false).
    #[serde(default)]
    pub can_request: bool,
}

/// A friend's live stream from one of your libraries (owner-facing; respects the listener's
/// scrobble privacy — private listeners simply don't appear).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ShareNowStreaming {
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub started_at: EpochMillis,
}

/// Per-grantee share + usage stats for the owner's library management screen ("who has access and
/// what are they doing with it"): the share itself plus request/download/listening activity
/// attributed to THIS library.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibraryShareStats {
    pub library_id: Uuid,
    pub grantee: PublicUser,
    pub permission_level: PermissionLevel,
    pub can_request: bool,
    pub created_at: EpochMillis,
    /// Acquisition requests this friend has queued into this library.
    pub requests_count: u32,
    /// How many of those completed (media actually landed in the library).
    pub downloads_completed: u32,
    /// Plays this friend has streamed from this library.
    pub plays: u32,
    /// Total listening time from this library, in milliseconds.
    pub ms_played: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played: Option<EpochMillis>,
    /// What they're streaming from this library right now, if anything (and not private).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub now_streaming: Option<ShareNowStreaming>,
}
