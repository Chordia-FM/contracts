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
    /// Stable, client-minted id for the device that sent this. Opaque to the Hub - it exists only so
    /// a client can tell ITS OWN report apart from one sent by another of the user's devices.
    ///
    /// One id per browser profile / app install, NOT per tab: tabs of the same browser already
    /// coordinate over `BroadcastChannel`, and giving them separate ids would make one person look
    /// like two devices playing at once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    /// Human label for that device ("Chrome on Android"). Display only, and only ever shown back to
    /// the user who sent it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_label: Option<String>,
}

/// The caller's OWN live now-playing entry.
///
/// This is what lets a signed-in browser say "playing on your phone" instead of "not playing": the
/// Hub already holds one ephemeral entry per user, and this hands the user back their own.
///
/// Not to be confused with [`crate::room::NowPlaying`], which is a listening-room broadcast, or with
/// [`FriendNowPlaying`], which is somebody else's.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DeviceNowPlaying {
    /// Which device reported it. `None` for a report that predates device ids - a client that
    /// cannot attribute an entry must assume it is its own rather than claim a phantom device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<Uuid>,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// When the report landed (epoch millis).
    pub started_at: EpochMillis,
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

/// One row in a followers / following list, carrying the viewer's relationship to that account so
/// the row can render its own follow button without a second round trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FollowUser {
    pub user: PublicUser,
    /// The viewer follows this account.
    pub viewer_follows: bool,
    /// This account follows the viewer.
    pub follows_viewer: bool,
    /// The viewer and this account are accepted friends.
    pub is_friend: bool,
}

/// One page of a followers / following list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FollowPage {
    pub items: Vec<FollowUser>,
    /// Total edges, so the header can show a count without walking every page.
    pub total: u32,
    /// Offset to request next. `None` when this is the last page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<u32>,
}

/// An artist on a user's profile "followed artists" shelf. Identity is the MBID because a followed
/// artist need not exist in this Hub's catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProfileArtist {
    pub artist_mbid: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Local catalog id when the artist resolves here, so the card can link internally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<Uuid>,
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
