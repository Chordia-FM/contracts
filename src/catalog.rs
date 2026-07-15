//! Catalog contracts: tracks, albums, artists, and the layered fingerprint that powers
//! cross-library "own-copy" matching.

use serde::{Deserialize, Serialize};

use crate::Uuid;

/// Layered identity for a recording, from strongest to weakest. Lets the same song match across
/// different files/encodings during own-copy lookup in DJ rooms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TrackFingerprint {
    /// AcoustID. A robust acoustic fingerprint that survives re-encoding. Preferred match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acoustid: Option<String>,
    /// MusicBrainz Recording ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_mbid: Option<String>,
    /// SHA-256 of the raw file bytes (hex). Used for exact-file matching and a bit-perfect integrity check.
    pub content_hash: String,
    /// Normalized metadata for the fuzzy fallback tuple.
    pub artist_norm: String,
    pub title_norm: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_norm: Option<String>,
    pub duration_ms: u32,
}

/// One artwork candidate for the metadata-editor art picker (from fanart.tv / Cover Art Archive).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtOption {
    /// Direct image URL at the provider.
    pub url: String,
    /// What the image is: `thumb` | `banner` | `background` | `logo` (artist) or `cover` (album).
    pub kind: String,
}

/// Available artwork for an artist or album. These are the choices shown in the art picker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtOptions {
    pub options: Vec<ArtOption>,
}

/// Audio stream characteristics, captured at scan time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AudioProperties {
    /// Container/codec, e.g. `flac`, `alac`, `wav`, `aac`, `mp3`, `eac3`.
    pub codec: String,
    pub sample_rate_hz: u32,
    pub bit_depth: u8,
    pub channels: u8,
    /// Whether this is a lossless source.
    pub lossless: bool,
    /// Spatial/Atmos source, flagged `passthrough_only` and never transcoded.
    pub spatial: bool,
    /// ReplayGain 2.0 track gain in dB (reference −18 LUFS), computed by the loudness pass. `None`
    /// until analyzed (or if ffmpeg/EBU R128 analysis is unavailable). The client applies it as a
    /// preamp when "Normalize volume" is on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gain_db: Option<f32>,
    /// Linear true-peak amplitude (≈0..1+), paired with `gain_db` to prevent clipping when applying
    /// the gain (the applied gain is capped at `1/peak`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak: Option<f32>,
}

/// A track DTO as served by a library's catalog API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Track {
    pub id: Uuid,
    pub library_id: Uuid,
    pub title: String,
    /// Track artist (may differ from album artist on compilations).
    pub artist: String,
    /// Album/compilation artist tag, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_artist: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    /// Release year extracted from tags.
    pub year: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    pub track_no: Option<u16>,
    pub disc_no: Option<u16>,
    /// Playback duration in milliseconds.
    pub duration_ms: u32,
    pub audio: AudioProperties,
    pub fingerprint: TrackFingerprint,
}

/// An album grouping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Album {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
    pub year: Option<u16>,
    pub track_count: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
}

/// An artist grouping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub album_count: u16,
}

/// Own-copy lookup query (`GET /v1/tracks/match`). Any subset may be provided; the library tries
/// from strongest to weakest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MatchQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acoustid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_norm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_norm: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
}

/// Own-copy lookup result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MatchResult {
    /// The matched local track, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track: Option<Track>,
    /// Which fingerprint layer produced the hit, for telemetry and UX confidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_on: Option<MatchStrength>,
}

/// Which identity layer produced an own-copy hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum MatchStrength {
    Acoustid,
    RecordingMbid,
    ContentHash,
    FuzzyMetadata,
}

// Catalog sync (library to Hub).
//
// By default a library pushes its catalog metadata and embedded artwork to the Hub, which becomes
// the source of truth for browsing and enriches it from external providers. (A library may opt to
// keep metadata local instead.) The Hub derives canonical artists/albums from the track rows.

/// One track in a catalog sync push. Carries everything the Hub needs to materialize canonical
/// artist/album/track rows and to associate the track with the originating library.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SyncTrack {
    pub title: String,
    pub artist: String,
    pub artist_normalized: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_normalized: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_artist: Option<String>,
    pub track_no: Option<u16>,
    pub disc_no: Option<u16>,
    pub year: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    pub duration_ms: u32,
    /// The library's own track id, used by the player to build the stream URL.
    pub track_ref: String,
    /// SHA-256 of the file bytes, the Hub's per-library track identity key.
    pub content_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isrc: Option<String>,
    /// SHA-256 of the embedded cover art, if any. Bytes are uploaded separately (see
    /// [`CatalogSyncResponse::missing_covers`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_hash: Option<String>,
    /// Edition qualifier ("Deluxe", "Special Edition", …) when this track comes from a deluxe/special
    /// edition that folds into the base album. `None` for the standard edition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Content advisory from the file's iTunes/ID3 rating tag: `"explicit"` or `"clean"`; `None` when
    /// unrated. Drives the EXPLICIT badge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory: Option<String>,
}

/// Body of `POST /v1/catalog/sync` (authenticated `Authorization: Library {server_api_key}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CatalogSyncRequest {
    /// The Hub-side library UUID this catalog belongs to.
    pub library_id: Uuid,
    pub tracks: Vec<SyncTrack>,
}

/// The Hub's canonical primary artist for one synced album, so the library can organize files on disk
/// under the same name the Hub shows (e.g. a collab tagged "Wiz Khalifa & MGK" resolves to "mgk", and
/// "Machine Gun Kelly"/"MGK" fold to "mgk"). Keyed by the album's normalized title + the album-artist
/// name the library sent, which together identify the library's album row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AlbumArtistResolution {
    pub album_normalized: String,
    /// The album-artist string the library sent (so it can match its own album row).
    pub album_artist: String,
    /// The Hub's canonical primary-artist name for that album.
    pub canonical_artist: String,
}

/// Response to a catalog sync: how many tracks were accepted and which referenced cover hashes the
/// Hub still needs the bytes for (upload via `PUT /v1/catalog/covers/{hash}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CatalogSyncResponse {
    pub accepted: u32,
    pub missing_covers: Vec<String>,
    /// The Hub's canonical album-artist per album in this batch, so the library can file albums under
    /// the same name (and relocate when it changes). Empty from older Hubs.
    #[serde(default)]
    pub canonical_album_artists: Vec<AlbumArtistResolution>,
}

/// Body of `POST /v1/catalog/prune` (server-authenticated). The authoritative set of track refs the
/// library still has; the Hub drops memberships for any of this library's tracks not in the set, so
/// files deleted on the library side disappear from browsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CatalogPruneRequest {
    /// The Hub-side library UUID this prune applies to.
    pub library_id: Uuid,
    /// Every `track_ref` the library currently has for this library.
    pub track_refs: Vec<String>,
}

// Browse DTOs (Hub to frontend) for the hierarchical library, artists, albums, and tracks views.

/// An artist as shown in a grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BrowseArtist {
    pub id: Uuid,
    /// MusicBrainz id, when known. Lets the Manager link an owned artist to its browse-all page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mbid: Option<String>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub album_count: u32,
    pub track_count: u32,
}

/// An album as shown in a grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BrowseAlbum {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<Uuid>,
    pub year: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    pub track_count: u32,
    /// MusicBrainz release-group MBID, when known. Lets the Manager link an owned album to its
    /// coverage/tracklist view (which is keyed by release-group).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mbid: Option<String>,
    /// First-release date (`YYYY-MM-DD`), when known. Used for precise newest-first ordering and
    /// labels in the discography (finer than `year`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    /// Primary release type (`Album`/`EP`/`Single`/`Compilation`/…), when known. Drives the
    /// discography type badges.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_type: Option<String>,
    /// Version tag: absent for the studio release, "instrumental" / "live" for a version release, so
    /// browsing can shelve them separately and Manager Discover can badge them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_type: Option<String>,
    /// True when this album is in the list only because the artist is a FEATURED credit on one of its
    /// tracks (not the album's primary artist) — lets the discography mark "appears on" entries.
    #[serde(default)]
    pub appears_on: bool,
}

/// A reference to one artist (for rendering a track's per-artist links).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistRef {
    pub id: Uuid,
    pub name: String,
}

/// A track row as shown in a list. `library_id` tells the player which library to stream from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BrowseTrack {
    pub id: Uuid,
    pub title: String,
    /// Display string of all credited artists (e.g. `"Drake feat. Rihanna"`).
    pub artist: String,
    /// Primary artist id (album attribution / "main artist" navigation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<Uuid>,
    /// All credited artists, ordered (primary first), for per-artist links. May be empty for
    /// legacy rows synced before multi-artist support.
    #[serde(default)]
    pub artists: Vec<ArtistRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_id: Option<Uuid>,
    pub track_no: Option<u16>,
    pub disc_no: Option<u16>,
    /// Edition qualifier ("Deluxe", …) when this track is from a deluxe/special edition folded into
    /// the base album; `None` for the standard edition. Lets the album view badge the extra tracks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    /// Content advisory: `"explicit"` or `"clean"` (from the file's iTunes/ID3 rating tag), `None`
    /// when unrated. Drives the EXPLICIT badge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory: Option<String>,
    pub duration_ms: u32,
    /// Total times this track has been played (across the Hub).
    #[serde(default)]
    pub plays: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    /// The library this playable track lives in (for grant + streaming).
    pub library_id: Uuid,
    /// The library's own track id, used to build the stream URL.
    pub track_ref: String,
    pub content_hash: String,
    /// When the track was added to the playlist / liked (epoch ms). None outside those contexts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_at: Option<crate::EpochMillis>,
}

/// One artist linked to another (an alias / side-project / band membership). Rendered as a
/// clickable chip on the artist page; each row is a real, independent artist (never a merge).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistRelation {
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Coarse bucket the UI groups by: `"alias"` (also-known-as / pseudonym), `"member"` (band
    /// membership, either direction), or `"related"` (collaboration / other).
    pub relation: String,
}

/// Full artist page payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistDetail {
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    /// Distinct listeners across the Hub in the last ~30 days.
    #[serde(default)]
    pub monthly_listeners: u32,
    // Enriched facts (MusicBrainz), all optional.
    /// Where the artist is from (associated area / country region name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    /// Formation place / birthplace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub begin_area: Option<String>,
    /// Career/life start, raw MB partial date (`YYYY` | `YYYY-MM` | `YYYY-MM-DD`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub begin_date: Option<String>,
    /// Career/life end (disbanded / deceased), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    /// True when the artist's life-span has ended (group disbanded / person deceased).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended: Option<bool>,
    /// MB type: `"Person"`, `"Group"`, `"Orchestra"`, …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_type: Option<String>,
    /// Linked artists such as aliases, side-projects, or bands (see [`ArtistRelation`]).
    #[serde(default)]
    pub related: Vec<ArtistRelation>,
    /// Labels this artist has released on, with the year span (derived from their albums).
    #[serde(default)]
    pub labels: Vec<ArtistLabel>,
    /// External links (official site, streaming, social) from MusicBrainz, curated to the platforms
    /// worth surfacing. Rendered as icon links on the artist page.
    #[serde(default)]
    pub links: Vec<ArtistLink>,
    pub albums: Vec<BrowseAlbum>,
    pub top_tracks: Vec<BrowseTrack>,
    /// Owned live material for this artist (live-version album tracks + scattered "(Live)" bonus
    /// variants), so the client can offer the dynamic Live album even when no live album exists.
    #[serde(default)]
    pub live_track_count: u32,
}

/// A synthesized per-artist "Live" collection: every owned live-version album track plus owned
/// live bonus variants scattered across the artist's other releases. Not a real `albums` row —
/// keyed on the artist (like `DailyMixDetail` is keyed on its seed), so it always reflects the
/// current library. The client localizes the display title/subtitle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LiveAlbum {
    pub artist_id: Uuid,
    pub artist_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Ordered by source album date, then disc/track position.
    pub tracks: Vec<BrowseTrack>,
    /// Distinct source albums the tracks were drawn from (for the localized subtitle).
    pub source_album_count: u32,
}

/// One external link for an artist. `kind` is a stable platform slug the UI maps to an icon:
/// `website`, `spotify`, `apple_music`, `youtube`, `youtube_music`, `soundcloud`, `bandcamp`,
/// `tidal`, `deezer`, `instagram`, `twitter`, `tiktok`, `facebook`, `wikipedia`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistLink {
    pub kind: String,
    pub url: String,
}

/// A label an artist has released on, with the span of years (for the artist-page label history).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistLabel {
    pub id: Uuid,
    pub name: String,
    /// Earliest / latest release year on this label (either may be None if no dates are known).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_year: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_year: Option<u16>,
    pub album_count: u32,
}

/// Full album page payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AlbumDetail {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<Uuid>,
    /// The primary artist's image (for the breadcrumb / header artist chip). Framing-aware.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    pub year: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    /// Canonical label display name (from the linked label entity, or a per-library override).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Canonical label id (for linking to the label page). None when unlabeled / override-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_type: Option<String>,
    /// Secondary release-group types (EP / Single / Compilation / Live / Remix / …).
    #[serde(default)]
    pub secondary_types: Vec<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    /// Total plays across all tracks on this album.
    #[serde(default)]
    pub plays: u32,
    pub tracks: Vec<BrowseTrack>,
}

/// A music label in the browse-by-label index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LabelSummary {
    /// None marks the synthetic "Unlabeled" bucket.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    /// Label logo (fanart.tv), if enriched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    pub album_count: u32,
}

/// A label page: its identity, enriched metadata, and the albums released on it (within accessible
/// libraries).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LabelDetail {
    /// None for the synthetic "Unlabeled" view.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mbid: Option<String>,
    // Enriched metadata (MusicBrainz, fanart.tv, and Wikipedia), all optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Founded / dissolved, raw MB partial dates (`YYYY` | `YYYY-MM` | `YYYY-MM-DD`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub founded: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defunct: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// MB label type: `"Production"`, `"Original Production"`, `"Imprint"`, …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_type: Option<String>,
    /// Former names merged into this canonical label (e.g. "Warner Bros. Records" became "Warner Records").
    #[serde(default)]
    pub former_names: Vec<String>,
    pub albums: Vec<BrowseAlbum>,
}

/// Global catalog search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SearchResults {
    pub artists: Vec<BrowseArtist>,
    pub albums: Vec<BrowseAlbum>,
    pub tracks: Vec<BrowseTrack>,
    #[serde(default)]
    pub labels: Vec<LabelSummary>,
}

/// A playlist summary (sidebar list).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub track_count: u32,
    /// User-set cover, if any. Takes precedence over `auto_cover_urls`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    /// Up to 4 distinct album covers from the playlist's tracks, for an auto mosaic when no
    /// `cover_url` is set.
    #[serde(default)]
    pub auto_cover_urls: Vec<String>,
    pub created_at: crate::EpochMillis,
    /// Whether the viewer owns this playlist (false = they collaborate on someone else's).
    #[serde(default = "default_true")]
    pub owned: bool,
    /// Whether the playlist has any collaborators.
    #[serde(default)]
    pub collaborative: bool,
}

fn default_true() -> bool {
    true
}

/// A playlist with its (playable) tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlaylistDetail {
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(default)]
    pub auto_cover_urls: Vec<String>,
    pub tracks: Vec<BrowseTrack>,
    /// Whether the viewer owns this playlist.
    #[serde(default = "default_true")]
    pub owned: bool,
    /// Whether the viewer may edit (add/remove tracks), as an owner or collaborator.
    #[serde(default = "default_true")]
    pub can_edit: bool,
    /// Users the owner has invited to collaborate.
    #[serde(default)]
    pub collaborators: Vec<crate::user::PublicUser>,
}
