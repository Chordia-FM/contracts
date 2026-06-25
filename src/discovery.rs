//! Discovery and recommendations contracts: daily mixes, "jump back in", trending, and similar
//! listeners. These power the Spotify-style home surface.

use serde::{Deserialize, Serialize};

use crate::catalog::{BrowseAlbum, BrowseArtist, BrowseTrack};
use crate::Uuid;

/// A daily mix opened as a playlist: its identity plus the generated track list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DailyMixDetail {
    pub seed_artist_id: Uuid,
    pub title: String,
    pub subtitle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub tracks: Vec<BrowseTrack>,
}

/// Globally trending content over a recent window (by distinct-listener count, then plays).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Trending {
    pub artists: Vec<BrowseArtist>,
    pub albums: Vec<BrowseAlbum>,
    pub tracks: Vec<BrowseTrack>,
}

/// A "Made for you" daily mix, a generated station anchored on an artist the user plays. Playing it
/// runs the same radio generation as the artist's radio (`/discovery/radio?artist=seed_artist_id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DailyMix {
    /// Display title, e.g. "Daily Mix 1".
    pub title: String,
    /// A few artist names featured in the mix, e.g. "Radiohead, Portishead and more".
    pub subtitle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Artist the mix is seeded from; play the mix by generating radio from this id.
    pub seed_artist_id: Uuid,
}

/// What kind of entity a "jump back in" card points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum RecentKind {
    Album,
    Artist,
    Playlist,
}

/// One "Jump back in" card: a recently played album/artist or one of the user's playlists.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RecentItem {
    pub kind: RecentKind,
    pub id: Uuid,
    pub name: String,
    /// Secondary line (e.g. the album's artist), when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// A user who shares listening taste with the caller (for the "similar listeners" surface).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SimilarUser {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Count of artists both the caller and this user have played.
    pub shared_artists: u32,
}
