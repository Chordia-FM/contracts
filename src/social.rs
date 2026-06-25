//! Social graph contracts: friendships and discovery.

use serde::{Deserialize, Serialize};

use crate::{user::PublicUser, EpochMillis, Uuid};

/// A client's report of what it's currently playing. It's ephemeral, held in memory on the Hub for
/// the live "Listening now" feed, never written to scrobble history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct NowPlayingReport {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<Uuid>,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// A friend's currently-playing track (for the live "Listening now" feed). Only friends whose
/// scrobble privacy allows it are included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FriendNowPlaying {
    pub user_id: Uuid,
    pub handle: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// When the track started (epoch millis).
    pub started_at: EpochMillis,
}

/// Lifecycle of a friendship edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum FriendshipStatus {
    /// Request sent, awaiting the other side.
    Pending,
    /// Mutual, so sharing and room invites are unlocked.
    Accepted,
    /// One side has blocked the other.
    Blocked,
}

/// A friendship edge as returned by the Hub. `user_a` is always the lexicographically smaller id
/// so each undirected edge has a single canonical row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Friendship {
    pub user_a: Uuid,
    pub user_b: Uuid,
    pub status: FriendshipStatus,
    /// Who initiated the pending request (for UI: incoming vs outgoing).
    pub requested_by: Uuid,
    pub created_at: EpochMillis,
}

/// Send a friend request by handle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FriendRequest {
    pub target_handle: String,
}

/// A discovery hit when searching for people to add.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DiscoveryResult {
    pub user: PublicUser,
    /// Existing relationship to the searcher, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship: Option<FriendshipStatus>,
}
