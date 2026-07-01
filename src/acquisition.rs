//! Acquisition contracts: quality profiles, the download request/job lifecycle (Hub ↔ frontend),
//! and the library-pulled job queue (Hub ↔ library). The Hub orchestrates; the library executes
//! (searches indexers, grabs torrents, imports files) and reports status back.

use serde::{Deserialize, Serialize};

use crate::{EpochMillis, Uuid};

// ── Quality profiles ─────────────────────────────────────────────────────────

/// A user's accepted-quality ranking. `allowed_formats` is best-first (e.g.
/// `["flac_hires","flac","mp3_320",…]`); the library scorer ranks candidates by this order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DownloadQualityProfile {
    pub id: Uuid,
    pub name: String,
    /// Ordered format keys, best first. Earlier = preferred.
    pub allowed_formats: Vec<String>,
    /// Stop hunting for an upgrade once a candidate at/above this format key is found. None = best.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cutoff: Option<String>,
    pub prefer_seeders: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_size_mb: Option<u32>,
    pub is_default: bool,
}

/// Create/update a quality profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DownloadQualityProfileInput {
    pub name: String,
    pub allowed_formats: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cutoff: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer_seeders: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_size_mb: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

// ── Download requests / jobs (Hub ↔ frontend) ────────────────────────────────

/// Ask the Hub to acquire something. `kind` is `album` | `track` | `discography`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DownloadRequestInput {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rg_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_mbid: Option<String>,
    /// Destination library. Required only when the user owns more than one acquisition-enabled lib.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_profile_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    /// When true, the library returns scored candidates for the user to pick instead of auto-grabbing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive: Option<bool>,
}

/// A download job as shown in the activity/queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DownloadJobView {
    pub id: Uuid,
    pub request_id: Uuid,
    pub library_id: Uuid,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seeders: Option<i32>,
    /// 0..1 download progress.
    pub progress: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: EpochMillis,
    pub updated_at: EpochMillis,
}

/// One status transition in a job's timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DownloadJobEvent {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub at: EpochMillis,
}

/// A scored torrent candidate for interactive selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DownloadCandidate {
    pub id: Uuid,
    pub guid: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seeders: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leechers: Option<i32>,
    pub rank: i32,
}

/// A job plus its event timeline and any candidates awaiting selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DownloadJobDetail {
    pub job: DownloadJobView,
    pub events: Vec<DownloadJobEvent>,
    pub candidates: Vec<DownloadCandidate>,
}

/// Redacted per-library acquisition health (the Hub's mirror of what the library reported).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibraryAcquisitionStatus {
    pub library_id: Uuid,
    pub enabled: bool,
    pub indexer_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reported_at: Option<EpochMillis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

// ── Library-pulled job queue (Hub ↔ library, server-authenticated) ───────────

/// Body of `POST /v1/manager/jobs/claim`. The library asks for up to `max` queued jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JobClaimRequest {
    pub server_id: Uuid,
    pub max: u32,
}

/// A claimed job with everything the library needs to execute it WITHOUT its own MusicBrainz access:
/// the resolved search hints (artist/album/year) and the quality profile to score against.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ClaimedJob {
    pub job_id: Uuid,
    pub library_id: Uuid,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rg_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    /// Pre-chosen release (set when the user picked a candidate for an interactive job). The library
    /// grabs this directly and skips the search/score step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_guid: Option<String>,
    pub interactive: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_profile: Option<DownloadQualityProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_name: Option<String>,
    /// The artist's name AS CREDITED IN THE RELEASE ERA (resolved Hub-side from the artist's
    /// date-ranged MusicBrainz aliases against the release year), e.g. "Machine Gun Kelly" for a
    /// 2019 album even though the artist is now "mgk". The library searches this FIRST (trackers
    /// indexed the release under the era name), falling back to `artist_name` (the current name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub era_artist_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    /// The album's expected track titles (from the Hub's MusicBrainz cache), so the library can verify
    /// the downloaded content actually IS this album before importing, and fail a mislabelled grab.
    /// Empty when the Hub has no cached tracklist (verification is then skipped).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_titles: Vec<String>,
}

/// Body of `POST /v1/manager/jobs/{id}/status`: a status transition reported by the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JobStatusUpdate {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_guid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seeders: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qbit_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// One scored candidate the library found (reported for interactive selection).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CandidateInput {
    pub guid: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seeders: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leechers: Option<i32>,
}

/// Body of `POST /v1/manager/jobs/{id}/candidates`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct JobCandidates {
    pub candidates: Vec<CandidateInput>,
}

/// Body of `POST /v1/manager/libraries/{id}/acquisition/report` — the library's health heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AcquisitionReport {
    pub enabled: bool,
    pub indexer_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// How often (days) to re-search a wanted release that hasn't been found yet, from this library's
    /// `[acquisition]` config. 0 = use the Hub default.
    #[serde(default)]
    pub research_interval_days: u32,
}

// ── Library direct interactive search (client → library, capability-authed) ──

/// Body of the library's `POST /v1/acquisition/search` — a live, scored Prowlarr search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AcquisitionSearch {
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
}

/// Body of the library's `POST /v1/acquisition/grab`. Grabs a chosen candidate into a library.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AcquisitionGrab {
    pub guid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rg_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
}
