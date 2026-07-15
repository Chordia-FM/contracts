//! User identity and profile contracts.

use serde::{Deserialize, Serialize};

use crate::streaming::QualityProfile;
use crate::{EpochMillis, Uuid};

/// A user's Last.fm connection status, for the settings "Connections" section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LastfmStatus {
    pub connected: bool,
    /// The connected Last.fm username, when connected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Body of `POST /v1/lastfm/session`: the single-use web-auth token from the Last.fm callback,
/// which the Hub exchanges (signed) for the user's permanent session key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LastfmSessionRequest {
    pub token: String,
}

/// Global account registration payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RegisterRequest {
    /// Unique, URL-safe handle (e.g. `nina`). Validated server-side.
    pub handle: String,
    pub email: String,
    pub password: String,
    pub display_name: String,
}

/// Full profile for the authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UserProfile {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub created_at: EpochMillis,
    /// Whether this user has site-admin access (the admin surface). Defaults false.
    #[serde(default)]
    pub is_admin: bool,
    /// Whether the account's email address has been confirmed. Defaults true so legacy/partial
    /// payloads don't nag verified users.
    #[serde(default = "default_true")]
    pub email_verified: bool,
    /// Whether two-factor (TOTP) auth is enabled on the account.
    #[serde(default)]
    pub totp_enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Profile fields the user can edit. Omitted fields are left unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// New unique handle. Validated server-side; omitted leaves the handle unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
}

/// Who may see a user's listening activity (recent-scrobble feed, taste compatibility). Defaults to
/// `Friends` so sharing is opt-in beyond one's friends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ScrobblePrivacy {
    /// Visible to anyone.
    Public,
    /// Visible only to accepted friends (the default).
    #[default]
    Friends,
    /// Visible to no one but the user.
    Private,
}

/// Per-user application preferences. Every field has a default so older/partial blobs still parse.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UserSettings {
    /// Listener-selected quality tier (the streaming ceiling). Serializes as `original`, `high`,
    /// `normal`, or `data_saver`.
    #[serde(default)]
    pub streaming_quality: QualityProfile,
    #[serde(default)]
    pub normalize_volume: bool,
    /// Whether listens are scrobbled to the Hub.
    #[serde(default = "yes")]
    pub scrobble: bool,
    /// Who can see this user's listening activity in social insights.
    #[serde(default)]
    pub scrobble_privacy: ScrobblePrivacy,
    /// Whether to receive transactional email notifications (e.g. a friend request). Default on.
    #[serde(default = "yes")]
    pub email_notifications: bool,
    /// Accent within the neon family: `pink`, `blue`, `purple`, or `green`.
    #[serde(default = "default_accent")]
    pub accent: String,
    /// Where the app opens by default: `app` or `library`.
    #[serde(default = "default_surface")]
    pub default_surface: String,
    /// Preferred UI + email language as a locale code (e.g. `en`, `es`). Empty = follow the
    /// request's `Accept-Language`. Drives server-originated text (errors, emails) and the client
    /// UI; the frontend also mirrors it into the `chordia_locale` cookie so SSR's first paint agrees.
    #[serde(default)]
    pub locale: String,
    #[serde(default = "yes")]
    pub autoplay: bool,
    /// How many upcoming queue tracks to prefetch in the background for seamless, gap-free
    /// playback. `0` disables prefetch; clamped client-side. Prefetched audio outside the window is
    /// evicted as the queue advances.
    #[serde(default = "default_preload")]
    pub preload_count: u32,
    /// Overlap-crossfade duration in seconds between consecutive tracks (`0` = off, the default, so
    /// tracks change with the normal short handoff). Clamped to `0..=12` client-side; streaming
    /// sources only (a crossfade needs the Web Audio graph). The frontend drives the dual-deck engine.
    #[serde(default)]
    pub crossfade_seconds: u32,
    /// Parametric equalizer state.
    #[serde(default)]
    pub eq: EqConfig,
    /// The user's saved custom EQ presets.
    #[serde(default)]
    pub eq_presets: Vec<EqPreset>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            streaming_quality: QualityProfile::default(),
            normalize_volume: false,
            scrobble: true,
            scrobble_privacy: ScrobblePrivacy::default(),
            email_notifications: true,
            accent: default_accent(),
            default_surface: default_surface(),
            locale: String::new(),
            autoplay: true,
            preload_count: default_preload(),
            crossfade_seconds: 0,
            eq: EqConfig::default(),
            eq_presets: Vec::new(),
        }
    }
}

fn default_preload() -> u32 {
    2
}

/// One parametric EQ band: a peaking filter at `freq` Hz with `gain` dB and quality factor `q`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EqBand {
    pub freq: f32,
    pub gain: f32,
    pub q: f32,
}

/// The live equalizer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EqConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Pre-amplifier gain in dB applied before the bands.
    #[serde(default)]
    pub preamp: f32,
    #[serde(default = "default_eq_bands")]
    pub bands: Vec<EqBand>,
}

impl Default for EqConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preamp: 0.0,
            bands: default_eq_bands(),
        }
    }
}

/// A named, saved equalizer preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EqPreset {
    pub name: String,
    #[serde(default)]
    pub preamp: f32,
    pub bands: Vec<EqBand>,
}

/// Default 10-band ISO graphic layout, all flat.
fn default_eq_bands() -> Vec<EqBand> {
    [
        31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
    ]
    .into_iter()
    .map(|freq| EqBand {
        freq,
        gain: 0.0,
        q: 1.4,
    })
    .collect()
}

fn yes() -> bool {
    true
}
fn default_accent() -> String {
    "pink".to_string()
}
fn default_surface() -> String {
    "app".to_string()
}

/// Minimal public view of a user. This is what friend discovery and room listings expose.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PublicUser {
    pub id: Uuid,
    pub handle: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}
