//! Pinned items: albums/artists/playlists a user pins to their sidebar (under Liked Songs).

use serde::{Deserialize, Serialize};

use crate::Uuid;

/// Which catalog entity a pin points at. `Radio` is a generated artist station (the pin's `id` is
/// the seed artist's id); it resolves to a "{Artist} Radio" entry that opens the station page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum PinKind {
    Album,
    Artist,
    Playlist,
    Radio,
}

/// A resolved pin (name + artwork) ready to render in the sidebar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PinnedItem {
    pub kind: PinKind,
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}
