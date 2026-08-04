//! Listening stations: a seed, a flavour, and a woven track list.
//!
//! Distinct from `discovery`'s `DailyMix`, which is a precomputed daily shelf. A station is
//! generated per request from a seed the listener chose, and its two flavours are genuinely
//! different products rather than two names for one list — see [`StationFlavour`].

use serde::{Deserialize, Serialize};

use crate::catalog::BrowseTrack;

/// What a station was seeded from. A genre seed is a slug, not an id — see [`Station::seed_id`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum StationKind {
    Artist,
    Track,
    Album,
    Genre,
    Playlist,
}

/// Which way a station faces.
///
/// **Not a label on the same list.** `Radio` points outward: it weights novelty, admits tracks the
/// listener has never played, and is endless. `Mix` points inward: it draws only from artists the
/// listener already plays plus their liked tracks, and is finite and stable for the day. The two are
/// the same machinery with the affinity term's SIGN flipped, which is the whole distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum StationFlavour {
    Radio,
    Mix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Station {
    pub kind: StationKind,
    /// The seed's identifier. A **String**, not a Uuid: a genre station is seeded by a slug.
    pub seed_id: String,
    /// The seed's own name, unformatted.
    ///
    /// Load-bearing rather than redundant with `title`. The server used to hard-code English titles
    /// (`format!("{name} Mix")`), which no amount of client-side i18n could undo. Handing back the
    /// raw name lets the client compose the title in the listener's language; `title` remains the
    /// server's fallback for any consumer that will not.
    pub seed_name: String,
    pub title: String,
    pub subtitle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub flavour: StationFlavour,
    pub tracks: Vec<BrowseTrack>,
    /// Opaque continuation token, or `None` when the station is finite.
    ///
    /// Stateless by construction: it encodes the day and an offset, so `/more` rebuilds the same
    /// weave rather than reading a server-side session, and a retried request is idempotent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}
