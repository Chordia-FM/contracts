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
    /// Global play count of the track.
    Plays,
    /// Whether the track is in the caller's Liked Songs.
    Liked,
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
    /// Greater-than-or-equal (numeric: year, plays).
    Gte,
    /// Less-than-or-equal (numeric).
    Lte,
    /// Boolean test (for `liked`); `value` is "true"/"false".
    Is,
}

/// One rule: `<field> <op> <value>`. `value` is always a string; the Hub parses it per field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartCondition {
    pub field: SmartField,
    pub op: SmartOp,
    pub value: String,
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
    Plays,
    Random,
}

/// The full rule set for a smart playlist.
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
