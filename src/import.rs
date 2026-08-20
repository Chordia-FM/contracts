//! Listening-history import (Spotify / Last.fm), a Super-Sonic capability.
//!
//! An import is a **job**, not a request: a Spotify extended streaming history runs to hundreds of
//! thousands of rows, so `POST /v1/me/imports` answers with a [`ImportJob`] immediately and the
//! parse/insert happens server-side while the client polls. The counters on the job advance batch
//! by batch, so the progress a client shows is real rather than a spinner.
//!
//! Idempotence is the load-bearing property. Every imported play gets a **deterministic** event id
//! derived from `(user, source, started_at, artist, title)`, so re-uploading the same export
//! inserts nothing and reports the rows as `duplicate_rows` instead of doubling a decade of
//! history.

use serde::{Deserialize, Serialize};

use crate::{EpochMillis, Uuid};

/// Which service an uploaded history file came from.
///
/// Detected from the file's content server-side; a client may also state it explicitly
/// (`POST /v1/me/imports?source=`), which is validated against the content rather than trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ImportSource {
    /// Spotify's "extended streaming history" JSON (either export vintage).
    Spotify,
    /// A Last.fm scrobble export CSV (`artist,album,track,timestamp`).
    Lastfm,
}

/// Lifecycle of an import job. Terminal states are `Done` and `Failed`; a client polls until it
/// reaches one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ImportJobStatus {
    Pending,
    Running,
    Done,
    Failed,
}

/// One history-import job, as created by `POST /v1/me/imports` and polled from
/// `GET /v1/me/imports/{id}`.
///
/// The counters partition `total_rows`: every row in the file ends up imported, a duplicate of an
/// already-stored event, or skipped (sub-30-second Spotify plays, podcast rows, unparseable rows).
/// `matched_rows` counts, among the imported, those resolved to a catalog track; the rest are
/// stored with their artist/title text and match later if that music arrives.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportJob {
    pub id: Uuid,
    pub source: ImportSource,
    pub status: ImportJobStatus,
    /// Rows found in the file. `0` until parsing finishes.
    pub total_rows: u32,
    /// Rows inserted as new listening events so far.
    pub imported_rows: u32,
    /// Rows that were already present (same deterministic event id), so nothing was inserted.
    pub duplicate_rows: u32,
    /// Imported rows resolved to a track in a library the owner can access.
    pub matched_rows: u32,
    /// Rows not worth importing: sub-30-second Spotify plays, podcast rows, unparseable rows.
    pub skipped_rows: u32,
    /// A stable failure code (`"malformed"`, `"internal"`, ...) when `status` is `Failed`; clients
    /// localize it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: EpochMillis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<EpochMillis>,
}

/// `GET /v1/me/imports`: the caller's recent import jobs, newest first.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ImportJobsResponse {
    pub jobs: Vec<ImportJob>,
}
