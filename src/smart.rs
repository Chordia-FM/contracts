//! Smart (rule-based, auto-updating) playlist contracts. A smart playlist stores a set of rules
//! that the Hub resolves to tracks on demand, so it stays current as the catalog and the user's
//! listening change.

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

/// A smart playlist summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartPlaylist {
    pub id: Uuid,
    pub name: String,
    pub created_at: EpochMillis,
    pub rules: SmartRules,
}

/// A smart playlist with its currently-resolved tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SmartPlaylistDetail {
    pub id: Uuid,
    pub name: String,
    pub rules: SmartRules,
    pub tracks: Vec<BrowseTrack>,
}
