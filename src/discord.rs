//! What a library's Discord bot asks the Hub for, with the server's own key.
//!
//! The bot plays a library into a voice channel where several people listen. Three things need
//! the Hub for that to feel like Chordia: knowing which listeners are Chordia users so a play can
//! land in *their* history, turning a library's track into the Hub's ids so a title can link to
//! its page, and an artist's picture, which only the Hub has.
//!
//! The trust rule, stated once: a library may attribute a play to a user only if that user could
//! have streamed the same track from that server in the web client — the server's owner, or a
//! holder of a share on any library of the server. Anyone else is simply absent from a resolve
//! answer, so the endpoint never says whether a Discord id belongs to a Chordia user.

use serde::{Deserialize, Serialize};

use crate::scrobble::ListeningEvent;
use crate::social::NowPlayingReport;
use crate::Uuid;

/// The most Discord ids one resolve call may carry.
pub const MAX_RESOLVE_IDS: usize = 100;
/// The most attributed events one ingest may carry.
pub const MAX_ATTRIBUTED_EVENTS: usize = 500;

/// `POST /v1/directory/listeners:resolve`: the Discord ids in a voice channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolveListenersRequest {
    pub discord_ids: Vec<String>,
}

/// A listener the server may attribute plays to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolvedListener {
    pub discord_id: String,
    pub user_id: Uuid,
    pub handle: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// The server's owner, as opposed to a share holder.
    pub is_owner: bool,
}

/// Only the listeners who passed the trust rule and have not opted out; the rest are absent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolveListenersResponse {
    pub listeners: Vec<ResolvedListener>,
}

/// One play, for one listener.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttributedEvent {
    pub user_id: Uuid,
    pub event: ListeningEvent,
}

/// `POST /v1/scrobbles:ingest-attributed`: plays the bot heard people listen to. The Hub re-checks
/// every `user_id` against the trust rule; the batch's `client_type` is forced to Discord.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AttributedScrobbleBatch {
    pub events: Vec<AttributedEvent>,
}

/// `POST /v1/directory/now-playing`: what the bot is playing to these listeners right now, or,
/// with no report, that it stopped. The Hub keeps the same live "listening now" entry a client
/// posts for itself, so a listener's profile shows the track. Every user is re-checked against
/// the trust rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ListenersNowPlaying {
    pub user_ids: Vec<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<NowPlayingReport>,
}

/// `POST /v1/catalog/resolve-tracks`: a library's own track ids, as it sent them in catalog sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolveTracksRequest {
    /// The Hub's id for the library (the one catalog sync uses).
    pub library_id: Uuid,
    pub track_refs: Vec<String>,
}

/// The Hub's ids for one of the library's tracks, for deep links.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolvedTrack {
    pub track_ref: String,
    pub track_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ResolveTracksResponse {
    pub tracks: Vec<ResolvedTrack>,
}

/// `POST /v1/catalog/artists:art`: artists by MusicBrainz id, else by normalized name (the same
/// normalization both sides use for `name_normalized`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistArtRequest {
    #[serde(default)]
    pub mbids: Vec<String>,
    #[serde(default)]
    pub names_normalized: Vec<String>,
}

/// An artist's page id and pictures. URLs are relative to the Hub, like every image URL it serves.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistArt {
    pub artist_id: Uuid,
    pub name: String,
    pub name_normalized: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mbid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ArtistArtResponse {
    pub artists: Vec<ArtistArt>,
}

/// `POST /v1/catalog/playlists:search`: playlists a server's bot may queue, by name: anyone's
/// public ones, and the asker's own (whatever their visibility) when the Hub knows who asked and
/// the server may act for them, by the same trust rule as attributing a play.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlaylistSearchRequest {
    pub query: String,
    /// The Discord account that asked, so their own playlists count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discord_id: Option<String>,
    /// At most this many, the owner's first; capped by the Hub.
    #[serde(default = "default_playlist_limit")]
    pub limit: u32,
}

fn default_playlist_limit() -> u32 {
    10
}

/// A playlist the bot may queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlaylistHit {
    pub id: Uuid,
    pub name: String,
    pub owner_handle: String,
    /// The asker's own; otherwise it is someone's public playlist.
    pub owned: bool,
    /// Every track on it, whether or not this server holds them.
    pub track_count: u32,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlaylistSearchResponse {
    pub playlists: Vec<PlaylistHit>,
}

/// `POST /v1/catalog/playlists:tracks`: a playlist's tracks as the refs of this server's own
/// libraries, in playlist order. Tracks the server does not hold are counted, not returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlaylistTracksRequest {
    pub playlist_id: Uuid,
    /// The Discord account that asked, so their own playlists may be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discord_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlaylistTrackRef {
    pub library_id: Uuid,
    /// The library's own id for the track.
    pub track_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlaylistTracksResponse {
    pub name: String,
    pub owner_handle: String,
    pub tracks: Vec<PlaylistTrackRef>,
    /// Tracks on the playlist that none of the server's libraries hold.
    pub missing: u32,
}
