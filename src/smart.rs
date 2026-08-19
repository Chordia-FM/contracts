//! Smart (rule-based, auto-updating) playlist contracts. A smart playlist stores a set of rules
//! that the Hub resolves to tracks and MATERIALISES as a snapshot, so every reader — page load,
//! queue, another device — sees the same list until it is refreshed.
//!
//! Refresh happens on three occasions: when the rules are saved, when the owner presses refresh
//! ([`SmartRefreshResult`] reports the diff), and on the owner's [`SmartPlaylist::refresh_interval_minutes`]
//! schedule. `0` on that field means never, i.e. manual-only.

use serde::{Deserialize, Serialize};

use crate::catalog::BrowseTrack;
use crate::{EpochMillis, Uuid};

/// The track attribute a condition tests.
///
/// The first seven are catalog facts. The rest describe the CALLER's relationship with the track —
/// when it arrived, when they last played it, how often — which is what makes rules like "on repeat
/// this month" or "liked but not played in a year" expressible at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SmartField {
    #[default]
    Artist,
    Title,
    Album,
    Genre,
    Year,
    /// Global play count of the track, across everyone on this Hub.
    Plays,
    /// Whether the track is in the caller's Liked Songs.
    Liked,
    /// Track length in milliseconds.
    Duration,
    /// When the track first appeared in one of the caller's libraries.
    AddedAt,
    /// When the caller last played it. Never played counts as "longer ago than any window".
    LastPlayed,
    /// When the caller first played it.
    FirstPlayed,
    /// The CALLER's play count, optionally scoped to a [`SmartPeriod`]. Distinct from `Plays`, which
    /// is the whole instance and is the same number for everyone.
    MyPlays,
    /// The album's release date, falling back to its year.
    ReleaseDate,
    /// The record label that released the album.
    Label,
    /// Whether the track is marked explicit.
    Explicit,
}

/// How a condition compares the field to its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SmartOp {
    #[default]
    Contains,
    Equals,
    /// Greater-than-or-equal (numeric: year, plays, duration).
    Gte,
    /// Less-than-or-equal (numeric).
    Lte,
    /// Boolean test (for `liked`, `explicit`); `value` is "true"/"false".
    Is,
    /// Date fields: strictly before `value` (a `YYYY-MM-DD` date).
    Before,
    /// Date fields: strictly after `value`.
    After,
    /// Date fields: within the last `value` days, counted back from now.
    InLast,
    /// Date fields: NOT within the last `value` days — which includes never. That inclusion is the
    /// point: "haven't played this in six months" has to match the songs never played at all.
    NotInLast,
    /// Inclusive range from `value` to `value2`. Both bounds are required.
    Between,
}

/// A window of listening history, for [`SmartField::MyPlays`] and period-scoped sorting.
///
/// Calendar and rolling windows are separate variants rather than a day count because they answer
/// different questions and read from different places: a calendar month or year is served by the
/// monthly rollup tables, while a rolling window has to scan the events themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SmartPeriod {
    /// One calendar month, `YYYY-MM`.
    Month { month: String },
    /// One calendar year.
    Year { year: u16 },
    /// The last N days, counted back from now on each refresh.
    Rolling { days: u32 },
}

/// One rule: `<field> <op> <value>`. `value` is always a string; the Hub parses it per field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartCondition {
    pub field: SmartField,
    pub op: SmartOp,
    pub value: String,
    /// Every value this rule accepts, matched as OR: `artist is A, B, C`. Empty means the rule is
    /// the single `value` above, which is how every rule written before this field existed reads.
    ///
    /// `value` is kept in sync with the first entry rather than being emptied, so a reader that
    /// predates this field still resolves the rule to something sensible instead of to nothing.
    /// Only the text fields use it; a numeric or date rule ignores it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    /// Upper bound for [`SmartOp::Between`]; ignored by every other operator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
    /// Scopes [`SmartField::MyPlays`] to a window. `None` means all time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period: Option<SmartPeriod>,
}

/// Whether all conditions must match (AND) or any (OR).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SmartMatch {
    #[default]
    All,
    Any,
}

/// Sort order for the resolved tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SmartSort {
    #[default]
    Title,
    /// Global play count.
    Plays,
    Random,
    /// The caller's own all-time play count.
    MyPlays,
    /// The caller's play count within [`SmartRules::sort_period`]. This is what makes a
    /// "top tracks of March" playlist come out in the right order.
    MyPlaysInPeriod,
    AddedAt,
    LastPlayed,
    ReleaseDate,
    Duration,
}

/// Sort direction. `None` on [`SmartRules::sort_dir`] means each sort's natural direction: A-Z for
/// title, largest or most recent first for everything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum SmartSortDir {
    Asc,
    Desc,
}

/// The full rule set for a smart playlist.
///
/// Every field added after the first version is `#[serde(default)]`, and the rules are stored as
/// JSONB — so a playlist saved by an older client still deserializes, unchanged, forever.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartRules {
    #[serde(default)]
    pub match_mode: SmartMatch,
    #[serde(default)]
    pub conditions: Vec<SmartCondition>,
    #[serde(default)]
    pub sort: SmartSort,
    /// Max tracks to resolve (clamped server-side). `None` = a sensible default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// The window [`SmartSort::MyPlaysInPeriod`] ranks within. Required by that sort and ignored by
    /// every other.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_period: Option<SmartPeriod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_dir: Option<SmartSortDir>,
}

/// `POST /v1/smart-playlists/preview`: what a rule set would produce, without saving it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartPreview {
    /// Matching tracks, ignoring [`SmartRules::limit`] so the builder can warn that a limit is
    /// cutting the result down.
    pub count: u32,
    /// The count stopped at an internal ceiling, so it is a floor rather than an exact number.
    #[serde(default)]
    pub count_capped: bool,
    /// The first few matches in sort order.
    pub sample: Vec<BrowseTrack>,
}

/// How often the Hub re-runs a playlist's rules on its own, in minutes. `0` is the explicit
/// "never" — the playlist then only changes when its owner edits the rules or presses refresh.
pub const SMART_REFRESH_NEVER: u32 = 0;
/// The interval a playlist gets when its owner never chose one.
pub const SMART_REFRESH_DEFAULT_MINUTES: u32 = 60;
/// Floor on a non-zero interval. Resolution is a catalog-wide scan, so a one-minute schedule would
/// buy nothing a manual press does not.
pub const SMART_REFRESH_MIN_MINUTES: u32 = 15;
/// Ceiling on a non-zero interval (one week). Past this, "never" is the honest setting.
pub const SMART_REFRESH_MAX_MINUTES: u32 = 10_080;

/// A smart playlist summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartPlaylist {
    pub id: Uuid,
    pub name: String,
    pub created_at: EpochMillis,
    pub rules: SmartRules,
    /// User-set cover, if any. Takes precedence over `auto_cover_urls`.
    ///
    /// Deliberately the same pair of fields a regular [`crate::catalog::Playlist`] carries, rather
    /// than the icon slug this used to be: a smart playlist is a playlist, and having one kind wear
    /// a picture and the other a glyph made them read as different species in the same sidebar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    /// Up to 4 distinct album covers from the current snapshot, for an auto mosaic when no
    /// `cover_url` is set. Follows the rules, so it re-shuffles as the playlist does.
    #[serde(default)]
    pub auto_cover_urls: Vec<String>,
    /// Minutes between automatic refreshes; [`SMART_REFRESH_NEVER`] (`0`) = manual only.
    #[serde(default)]
    pub refresh_interval_minutes: u32,
    /// When the snapshot was last successfully rebuilt. `None` = never resolved yet. A FAILED
    /// refresh does not move this, so a stale stamp is a visible symptom rather than a lie.
    #[serde(default)]
    pub refreshed_at: Option<EpochMillis>,
    /// Tracks in the current snapshot, without hydrating them.
    #[serde(default)]
    pub track_count: u32,
}

/// A smart playlist with its materialised tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartPlaylistDetail {
    pub id: Uuid,
    pub name: String,
    pub rules: SmartRules,
    pub tracks: Vec<BrowseTrack>,
    /// See [`SmartPlaylist::cover_url`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    /// See [`SmartPlaylist::auto_cover_urls`].
    #[serde(default)]
    pub auto_cover_urls: Vec<String>,
    /// Minutes between automatic refreshes; [`SMART_REFRESH_NEVER`] (`0`) = manual only.
    #[serde(default)]
    pub refresh_interval_minutes: u32,
    /// When the snapshot was last successfully rebuilt. `None` = never resolved yet.
    #[serde(default)]
    pub refreshed_at: Option<EpochMillis>,
}

/// What one refresh actually changed. Returned by the manual refresh so the button can say
/// something true — including "nothing changed", which is a result and not a failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartRefreshResult {
    /// Tracks now in the playlist that were not in the previous snapshot.
    pub added: u32,
    /// Tracks that were in the previous snapshot and no longer match.
    pub removed: u32,
    /// Size of the new snapshot.
    pub total: u32,
    pub refreshed_at: EpochMillis,
}
