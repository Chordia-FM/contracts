//! Lyrics contracts: synced/unsynced lyric lines served from the Hub's DB-backed cache.
//!
//! Lyrics originate from an external provider (LRCLIB) but are normalized and persisted as track
//! metadata, so the wire shape here is provider-agnostic. Users can also override them manually
//! (see [`LyricsEditInput`]).

use serde::{Deserialize, Serialize};

use crate::Uuid;

/// Whether the lyric lines carry per-line timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum LyricsSyncType {
    /// Plain lyrics with no timing - render statically.
    Unsynced,
    /// Each line has a start/end time and can be highlighted in sync with playback.
    LineSynced,
}

/// One lyric line. `start_ms`/`end_ms` are present only for [`LyricsSyncType::LineSynced`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LyricsLine {
    pub text: String,
    /// Line start offset from the top of the track, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_ms: Option<u32>,
    /// Line end offset, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_ms: Option<u32>,
}

/// Lyrics for a single track, as served by `GET /v1/lyrics/{track_id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Lyrics {
    pub track_id: Uuid,
    pub sync_type: LyricsSyncType,
    pub lines: Vec<LyricsLine>,
    /// YouTube video id captured alongside the lyrics, for future video support. May be absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub youtube_video_id: Option<String>,
    /// True when a user set/edited these lyrics by hand (the auto-fetcher won't overwrite them).
    #[serde(default)]
    pub manually_edited: bool,
}

/// User-supplied lyrics edit (`PUT /v1/lyrics/{track_id}`). The body is raw text the server parses:
/// when `synced`, each line is LRC (`[mm:ss.xx] words`); otherwise lines are plain text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LyricsEditInput {
    /// Whether `text` is timed LRC (`true`) or plain lines (`false`).
    pub synced: bool,
    /// The lyrics body: LRC lines when `synced`, else one plain line per newline.
    pub text: String,
}
