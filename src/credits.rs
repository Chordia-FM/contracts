//! Who actually made the record: producers, writers, engineers, featured performers.
//!
//! Distinct from the *credited artists* on a track ([`crate::catalog`]'s artist line, "X feat. Y"),
//! which is the display name of the release and lives in `track_artists`. These are the liner
//! notes — the people a listener goes looking for when a snare sounds a particular way, and the
//! only place a mix engineer is ever named.
//!
//! ## The Hub owns the merged result
//!
//! Three sources, and the Hub is the one place that sees all three:
//!
//! - **MusicBrainz** — recording-level artist relationships (`producer`, `mix`, `vocal`,
//!   `instrument`, …). Broad but shallow: great for well-catalogued releases, empty for most
//!   others.
//! - **The library's sidecar** — a `… Tracklist.txt` beside an album, which downloaders leave and
//!   which carries per-track personnel far richer than any tag. The library parses it and *sends*
//!   it here; it keeps its own copy so it can answer with no Hub at all, and so a hub-less
//!   deployment is a supported one rather than a degraded one.
//! - **Manual** — a correction, which by definition outranks anything a machine derived.
//!
//! Precedence is per `(name, role)` rather than wholesale by source: MusicBrainz names the mix
//! engineer that a sidecar omits, and the sidecar names the session players MusicBrainz never
//! catalogued. Taking one source entire would throw away half of what is known. Where the same
//! person in the same role appears twice, the higher-precedence spelling and artist link win.

use serde::{Deserialize, Serialize};

use crate::Uuid;

/// Where a credit came from, in precedence order — later variants win a collision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum CreditSource {
    /// Derived by the Hub's enrichment worker from MusicBrainz recording relationships.
    Musicbrainz,
    /// Parsed by a library from a `… Tracklist.txt` sidecar and pushed here on catalog sync.
    LibrarySidecar,
    /// Entered by a person. Outranks everything, and survives every re-enrichment.
    Manual,
}

/// One person or organisation credited in one role.
///
/// A person credited for three things is three [`Credit`]s and not one with three roles, because
/// the panel groups by role: a reader looks for "who produced this", not "what did this person do".
/// The library's own parse keeps the other shape (name → roles) since a text file lists it that
/// way; the flattening happens on the way here.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Credit {
    pub name: String,
    /// Free text, as the source phrased it — "Producer", "Mix Engineer", "Bass". Deliberately not
    /// an enum: MusicBrainz alone has well over a hundred relationship types with attributes on
    /// top, and a closed set would silently drop every role it had not anticipated.
    pub role: String,
    /// The catalog artist this credit resolves to, when the Hub already knows the person. Makes the
    /// name a link; `None` is the common case for session players who have no releases of their own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<Uuid>,
    /// A company rather than a person (a label, a studio) — rendered without an avatar.
    #[serde(default)]
    pub is_org: bool,
    pub source: CreditSource,
}

/// Every credit sharing one role, in the order the source listed them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreditGroup {
    pub role: String,
    pub credits: Vec<Credit>,
}

/// Response of `GET /v1/tracks/{id}/credits`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TrackCredits {
    pub track_id: Uuid,
    pub groups: Vec<CreditGroup>,
}

/// Response of `GET /v1/albums/{id}/credits` — the union across the album's tracks.
///
/// An aggregate rather than a per-track breakdown: an album's producer is nearly always the same
/// name repeated down every track, and printing it forty times is not liner notes, it is a wall.
/// The per-track view is one request away on the track itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AlbumCredits {
    pub album_id: Uuid,
    pub groups: Vec<CreditGroup>,
    /// How many of the album's tracks contributed at least one credit, so the panel can say
    /// "from 9 of 12 tracks" instead of implying the album is fully documented when it is not.
    pub tracks_with_credits: u32,
    pub track_count: u32,
}

/// One credit as a library parsed it, on its way to the Hub inside a catalog sync.
///
/// Flat `(name, role)` like [`Credit`], and without a source: the wire it arrives on *is* the
/// source. No `artist_id` either — a library has no opinion about the Hub's artist ids.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SyncCredit {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub is_org: bool,
}
