//! Insights / "Wrapped" contracts. The output of the centralized analytics engine.

use serde::{Deserialize, Serialize};

use crate::user::PublicUser;
use crate::{EpochMillis, Uuid};

/// Aggregation window for a stats report. Mirrors the Last.fm-style windows: 7-day, 1/3/6/12
/// month, and all-time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum Period {
    /// Trailing 7 days.
    Week,
    /// Trailing 30 days (~1 month).
    Month,
    /// Trailing 90 days (~3 months).
    Quarter,
    /// Trailing 180 days (~6 months).
    HalfYear,
    /// Trailing 365 days (~12 months).
    Year,
    /// All recorded history.
    Overall,
}

/// Grain the over-time series is bucketed at, so the client can label its x-axis correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum BucketGranularity {
    Day,
    Month,
}

/// One bucket in a time series of listening activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TimeBucket {
    /// Inclusive start of the bucket (epoch millis, UTC-truncated).
    pub start: EpochMillis,
    pub plays: u32,
    pub ms_played: u64,
}

/// Chart-oriented listening data for a period: an activity time series plus the listening-clock
/// (hour-of-day) and weekday distributions. Computed live from the partitioned fact table, scoped
/// to one user over a bounded window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ListeningCharts {
    pub period: Period,
    pub window_start: EpochMillis,
    pub window_end: EpochMillis,
    pub granularity: BucketGranularity,
    /// Plays bucketed across the window, chronological. Daily for windows up to a year; monthly for
    /// `Overall`. Doubles as a calendar-heatmap source at day granularity.
    pub over_time: Vec<TimeBucket>,
    /// Plays by hour-of-day (UTC), exactly 24 entries (index = hour 0 to 23).
    pub clock: Vec<u32>,
    /// Plays by day-of-week (UTC), exactly 7 entries (index 0 = Sunday through 6 = Saturday).
    pub weekday: Vec<u32>,
}

/// One entry in the full scrobble history feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HistoryEntry {
    /// The play event's idempotency id (also the keyset-pagination tiebreak).
    pub event_id: Uuid,
    /// Resolved catalog track id, if the play matched the catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<Uuid>,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub played_at: EpochMillis,
    pub ms_played: u64,
}

/// A page of scrobble history, newest first, with a keyset cursor for the next page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HistoryPage {
    pub entries: Vec<HistoryEntry>,
    /// Pass these back as `before_ms` / `before_id` to fetch the next page. Both `None` at the end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_before_ms: Option<EpochMillis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_before_id: Option<Uuid>,
}

/// One ranked entry in a top-N list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TopItem {
    /// Catalog id of the artist/track/album.
    pub id: Uuid,
    pub name: String,
    pub plays: u32,
    pub ms_played: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// A listening-insights report for a user over a period. Served from precomputed rollups, never
/// from the raw fact table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WrappedReport {
    pub user_id: Uuid,
    pub period: Period,
    /// Inclusive start of the window (epoch millis).
    pub window_start: EpochMillis,
    /// Exclusive end of the window (epoch millis).
    pub window_end: EpochMillis,
    pub total_plays: u32,
    pub total_ms_played: u64,
    pub unique_tracks: u32,
    pub unique_artists: u32,
    pub top_artists: Vec<TopItem>,
    pub top_tracks: Vec<TopItem>,
    pub top_albums: Vec<TopItem>,
    /// Top genres by play count. `id` is a stable hash of the genre name (genres aren't catalog
    /// entities), and `image_url` is always `None`.
    pub top_genres: Vec<TopItem>,
}

/// Which kind of catalog entity a per-entity stats query is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum EntityKind {
    Artist,
    Album,
    Track,
}

/// A user's personal listening stats for one catalog entity (artist/album/track). This is the data
/// behind the "your stats" panels on the catalog detail pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EntityStats {
    pub kind: EntityKind,
    pub id: Uuid,
    pub total_plays: u32,
    pub total_ms_played: u64,
    /// First/last time the user played this entity (epoch millis). `None` if never played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_played: Option<EpochMillis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played: Option<EpochMillis>,
    /// 1-based rank among the user's entities of this kind, by all-time play count. `None` if the
    /// user has never played it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// Monthly play trend (chronological), for a sparkline.
    pub trend: Vec<TimeBucket>,
}

/// One entry in the friends' recent-activity feed, a play by a friend whose privacy allows it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FriendScrobble {
    pub user_id: Uuid,
    pub handle: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub played_at: EpochMillis,
}

/// Taste-compatibility between the caller and another user: a 0 to 1 overlap score plus the artists
/// they share. Only returned when the other user's privacy permits it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Compatibility {
    pub user_id: Uuid,
    pub handle: String,
    pub display_name: String,
    /// Cosine similarity (0.0 to 1.0) over the two users' top-artist play vectors.
    pub score: f32,
    /// Artists both users have played, most-shared first (capped).
    pub shared_artists: Vec<TopItem>,
}

/// A user's shareable public listening profile. Listening stats are populated only when the viewer
/// is allowed to see them (per the target's scrobble privacy); otherwise `private` is true and the
/// lists are empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PublicProfile {
    pub user: PublicUser,
    /// When the account was created (epoch millis).
    pub created_at: EpochMillis,
    /// True when the viewer may not see this user's listening activity.
    pub private: bool,
    pub total_plays: u32,
    pub top_artists: Vec<TopItem>,
    pub top_tracks: Vec<TopItem>,
    pub recent: Vec<RecentPlay>,
}

/// Lightweight "recently played" feed item for the home view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RecentPlay {
    pub track_id: Uuid,
    pub title: String,
    pub artist: String,
    pub played_at: EpochMillis,
}
