//! Manager contracts: library-coverage analytics and "what am I missing" detection.
//!
//! The Manager is Chordia's Lidarr/Overseerr-style layer. It compares what a user OWNS (the Hub
//! catalog, scoped to their libraries) against an artist's full discography (cached from
//! MusicBrainz in the Hub `ext_*` tables) to surface coverage percentages and missing albums /
//! tracks. Later phases add browse-all, follows, and torrent-based acquisition; their contracts are
//! added to this module as they land.

use serde::{Deserialize, Serialize};

use crate::catalog::BrowseAlbum;
use crate::{EpochMillis, Uuid};

/// Headline coverage across the libraries a user owns (and optionally those shared with them).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CoverageSummary {
    /// Album-level coverage: owned release-groups / total release-groups across touched artists
    /// (0..100). The headline number.
    pub album_pct: f32,
    /// Coarse artist-level coverage: artists fully owned / artists touched (0..100).
    pub artist_pct: f32,
    /// Release-groups owned across all touched artists (an artist is "touched" if the user owns at
    /// least one of their tracks).
    pub owned_rgs: u32,
    /// Total release-groups across those artists (from the discography cache).
    pub total_rgs: u32,
    /// Artists with at least one owned track that the discography cache has been fetched for.
    pub touched_artists: u32,
    /// Of those, how many are fully owned (no missing release-groups).
    pub complete_artists: u32,
    /// Artists still awaiting a discography fetch (coverage for them is provisional until fetched).
    pub pending_artists: u32,
    /// Per-library owned counts, in priority order.
    pub per_library: Vec<LibraryCoverage>,
    /// Libraries the user has excluded from this computation.
    pub excluded_library_ids: Vec<Uuid>,
    /// Whether shared-with-me libraries were counted (mirrors [`ManagerPrefs::include_shared`]).
    pub include_shared: bool,
}

/// Owned counts for one library in the coverage scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibraryCoverage {
    pub library_id: Uuid,
    pub name: String,
    /// True when this library is owned by the user (false = shared-with-them).
    pub owned: bool,
    /// Whether this library is currently excluded from coverage math.
    pub excluded: bool,
    pub track_count: u32,
    pub album_count: u32,
    pub artist_count: u32,
}

/// Per-user Manager preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ManagerPrefs {
    /// Count shared-with-me libraries toward coverage.
    #[serde(default)]
    pub include_shared: bool,
    /// Default destination library for downloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_library_id: Option<Uuid>,
    /// Default quality profile id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_quality_profile_id: Option<Uuid>,
    /// One-time acknowledgement that the user is responsible for content legality (gates downloads).
    #[serde(default)]
    pub acq_ack: bool,
}

/// Replace the set of libraries excluded from coverage (body of `PUT /v1/manager/coverage/exclusions`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExclusionsUpdate {
    pub library_ids: Vec<Uuid>,
}

/// A release-group from an artist's discography cache (may be owned or missing).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExtReleaseGroup {
    /// MusicBrainz release-group MBID.
    pub mbid: String,
    pub title: String,
    /// MusicBrainz primary-type: `"Album"` | `"EP"` | `"Single"` | …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_type: Option<String>,
    /// secondary-types (`"Live"`, `"Compilation"`, …) the UI can filter on.
    #[serde(default)]
    pub secondary_types: Vec<String>,
    /// First release date, ISO `YYYY-MM-DD` (may be a partial date padded to the 1st).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_release_date: Option<String>,
    /// Cover Art Archive cover, if fetched (Hub-relative image URL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_total: Option<u32>,
}

/// Owned-vs-missing breakdown for one artist the user (partially) owns.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistCoverage {
    /// The owned-catalog artist id (for navigation).
    pub artist_id: Uuid,
    pub name: String,
    /// The artist's MusicBrainz id, if known (needed to diff the discography).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_mbid: Option<String>,
    /// Albums the user owns (from their libraries).
    pub owned: Vec<BrowseAlbum>,
    /// Release-groups in the discography the user does NOT own.
    pub missing: Vec<ExtReleaseGroup>,
    /// Album-level coverage for this artist (0..100).
    pub coverage_pct: f32,
    /// True when the discography is still being fetched (missing list is provisional). The client
    /// should refetch on the next realtime catalog nudge.
    pub refreshing: bool,
}

/// One recording (track) in a cached release-group's listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExtRecording {
    pub recording_mbid: String,
    pub title: String,
    pub disc_no: u16,
    pub position: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length_ms: Option<u32>,
    /// Whether the user owns this track (matched by recording MBID, or title + duration ±2s).
    #[serde(default)]
    pub owned: bool,
}

// ── Phase 2: browse-all discovery + follows ──────────────────────────────────

/// An artist surfaced by browse-all discovery, with whether the user already owns them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DiscoverArtist {
    pub mbid: String,
    pub name: String,
    /// MusicBrainz disambiguation comment (e.g. "UK rock band"), to tell same-named artists apart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disambiguation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Genres/tags (cached from MusicBrainz/Last.fm), for a "who is this" subtitle on the card.
    #[serde(default)]
    pub genres: Vec<String>,
    /// True when the user owns this artist in their accessible libraries.
    pub owned: bool,
    /// The owned-catalog artist id (set only when owned), for linking into the owned artist page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_artist_id: Option<Uuid>,
    /// Whether the user already follows this artist.
    pub following: bool,
}

/// A release-group surfaced by discovery / an artist's full discography, owned-tagged.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DiscoverReleaseGroup {
    pub mbid: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_type: Option<String>,
    #[serde(default)]
    pub secondary_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_release_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    /// "instrumental" / "live" version release (from the MB disambiguation / the `Live` secondary type),
    /// so the discography can badge it and shelve it apart. Absent for the studio release.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_type: Option<String>,
    pub owned: bool,
}

/// Results of a browse-all search: matching artists and release-groups, owned-first.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DiscoverResults {
    pub artists: Vec<DiscoverArtist>,
    pub release_groups: Vec<DiscoverReleaseGroup>,
}

/// A not-necessarily-owned artist's page in discovery: identity + full discography, owned-overlaid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExtArtistDetail {
    pub mbid: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disambiguation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Short artist biography (cached from Last.fm), shown on the external artist page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// Genres/tags (cached from MusicBrainz/Last.fm).
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_artist_id: Option<Uuid>,
    /// The catalog `artists` row this discovered artist resolves to (created on demand). Editable via
    /// the normal admin metadata tools; this is how Manager artist metadata is modified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_artist_id: Option<Uuid>,
    pub following: bool,
    /// True while the discography is still being fetched from MusicBrainz (list is provisional).
    pub refreshing: bool,
    pub release_groups: Vec<DiscoverReleaseGroup>,
}

/// A followed artist, as shown in the follows list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FollowedArtist {
    pub artist_mbid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_artist_id: Option<Uuid>,
    pub auto_download: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_library_id: Option<Uuid>,
    #[serde(default)]
    pub monitor_types: Vec<String>,
    pub created_at: EpochMillis,
}

/// Follow an artist / update a follow (body of `POST /v1/manager/follows`, `PATCH .../{mbid}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FollowInput {
    pub artist_mbid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_download: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_library_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_profile_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor_types: Option<Vec<String>>,
}

/// One edition of an album (Standard, Deluxe, Expanded, Remaster, …) with its own tracklist and
/// owned/missing breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AlbumEdition {
    /// The representative release MBID for this edition.
    pub release_mbid: String,
    /// Display label, e.g. "Standard", "Special edition", "2020 remaster".
    pub label: String,
    /// True when this edition is a remaster (same songs, different master), shown independently.
    pub is_remaster: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    /// Total tracks in this edition.
    pub total: u32,
    /// How many of them the user owns.
    pub owned_count: u32,
    /// This edition's tracklist in order, each flagged `owned`.
    pub tracks: Vec<ExtRecording>,
}

/// Every edition of one album (release-group), each with its own tracklist + owned/missing breakdown.
/// This is the Manager's album view. Works for an album the user owns (to show which tracks/editions are
/// missing) and for one they don't own at all (everything missing).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AlbumTrackCoverage {
    /// The release-group MBID this breakdown is for.
    pub rg_mbid: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    /// The album artist's MusicBrainz id, for download context / linking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_mbid: Option<String>,
    /// The album artist's display name, for the breadcrumb trail and linking back to the artist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_name: Option<String>,
    /// The album artist's image, for the breadcrumb avatar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    /// The album's editions, in display order (Standard first).
    pub editions: Vec<AlbumEdition>,
    /// True while the recording listing is still being fetched.
    pub refreshing: bool,
}
