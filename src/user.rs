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

/// Who may see one profile surface. Used for the per-surface visibility knobs in
/// [`UserSettings`].
///
/// **A ladder, and each rung includes the ones below it**: `Private` < `Friends` < `Followers` <
/// `Public`. So `Followers` admits followers *and* friends, and `Public` admits anyone signed in.
///
/// The order is deliberate and is the opposite of set size in the underlying graph. Friendship is
/// mutual consent and following is one-directional, so neither group strictly contains the other —
/// someone can follow you without being a friend, and a friend need never have followed you. Placing
/// `Friends` tighter treats the closer relationship as the narrower audience, which is what the
/// setting reads as, and matches how the older `scrobble_privacy` `friends` value already behaves.
///
/// Anything wider is an explicit choice: the default is `Private`.
///
/// `Public` means *any signed-in account*, not the open internet — every one of these surfaces sits
/// behind authentication.
///
/// Shape-similar to [`ScrobblePrivacy`], which has no `Followers` rung; see the note there for why
/// the two must not be merged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum Audience {
    /// Visible to no one but the user. The default — anything wider is opted into.
    #[default]
    Private,
    /// Visible to accepted (mutual) friends.
    Friends,
    /// Visible to anyone who follows the user, and to friends.
    Followers,
    /// Visible to any signed-in account.
    Public,
}

/// One external link on a user's profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProfileLink {
    /// Free-text kind (e.g. `bandcamp`, `website`), used to pick an icon. Unknown kinds render generically.
    pub kind: String,
    pub url: String,
}

/// What a Hub says about itself to a client that has not signed in — capability flags, plus enough
/// identity for the desktop app's hub picker.
///
/// Unauthenticated by necessity: the desktop app calls this on a URL the user has just typed, to
/// answer "is this a Chordia Hub, what do we call it, and does it do Discord" before there is any
/// account here to authenticate with. A 200 with this shape is itself the "yes, this is a Hub"
/// check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct InstanceInfo {
    /// Whether the follow graph and public profiles are enabled on this instance.
    pub social_enabled: bool,
    /// Operator-facing name, shown in the desktop hub picker. Self-hosters run several, and "Hub"
    /// three times is not a picker.
    pub name: String,
    /// Hub software version, for support and for a client that wants to warn about a stale server.
    pub version: String,
    /// The Hub's canonical **web** frontend. The desktop app anchors the links its users copy here,
    /// because its own origin (`https://tauri.localhost`) means nothing to whoever receives one.
    pub frontend_url: String,
    /// Whether "Continue with Discord" will work here. False when the operator never configured a
    /// Discord app, and a button that leads to a 500 is worse than no button.
    pub discord_oauth: bool,
    /// This deployment's Discord application id, when it has one.
    ///
    /// Also its Rich Presence application id — Discord has one identifier for both, so the desktop
    /// app publishes presence under whichever application the operator already configured for
    /// sign-in rather than carrying a second one baked in at build time.
    ///
    /// Public because it already is: this value appears in the query string of every OAuth
    /// authorize URL a browser visits. The client *secret* is the confidential half and stays on
    /// the Hub. Derived from the same config as `discord_oauth` above, so the two cannot disagree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discord_client_id: Option<String>,
    /// The accent this deployment wears before a listener has chosen their own — a preset name or a
    /// `#rrggbb`. `None` means the operator has not set one and the app's own default stands, which
    /// is different from their choosing that colour: it keeps following the default if it changes.
    ///
    /// Public because it themes the sign-in and landing screens, which nobody is signed in to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
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
    /// Short free-text profile bio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// Content hash of an uploaded banner image (from the Hub image endpoint), not a URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_hash: Option<String>,
    /// Full replacement list of profile links. Omitted leaves the existing links unchanged; an
    /// empty vec clears them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<ProfileLink>>,
}

/// Who may see a user's listening activity (recent-scrobble feed, top artists, taste
/// compatibility). Defaults to `Friends` so sharing is opt-in beyond one's friends.
///
/// **This IS the listening-history / top-artists visibility control** — enforced in
/// `backend/src/api/v1/insights.rs`. Do not add a separate `history_visibility` knob.
///
/// The duplication with [`Audience`] is DELIBERATE and the two must not be merged: this enum's TS
/// binding and its `scrobble_privacy` key in the stored `user_settings` JSONB payload are
/// load-bearing across the frontend and in SQL (`payload->>'scrobble_privacy'`).
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
    /// Accent preset name, hue-ordered: `crimson`, `ember`, `amber`, `lime`, `green`, `teal`,
    /// `blue`, `indigo`, `purple`, `magenta`, `pink`, or `default` — which is the default, and means
    /// "follow this deployment's accent" (`InstanceInfo::accent`) rather than naming a colour. An
    /// unrecognized value is used verbatim as a CSS colour.
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
    /// IANA timezone used for calendar-based listening insights. Empty = follow the client.
    #[serde(default)]
    pub timezone: String,
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
    /// Who may see the user's profile page at all.
    ///
    /// Defaults to `Private`. This is the front door — every other surface here is only reachable
    /// once it has let the viewer through — so it is the one that has to be opened deliberately.
    #[serde(default = "default_audience_private")]
    pub profile_visibility: Audience,
    /// Who may see the list of accounts following this user.
    #[serde(default = "default_audience_friends")]
    pub followers_visibility: Audience,
    /// Who may see the list of accounts this user follows.
    #[serde(default = "default_audience_friends")]
    pub following_visibility: Audience,
    /// Who may see this user's public playlists on their profile. Defaults to `Private`: a
    /// playlist's own `PlaylistVisibility` opts it in, this only widens the *shelf*.
    #[serde(default = "default_audience_private")]
    pub playlists_visibility: Audience,
    /// Who may see the artists this user follows.
    #[serde(default = "default_audience_friends")]
    pub followed_artists_visibility: Audience,
    /// Whether other users may follow this account without asking. Default on.
    #[serde(default = "yes")]
    pub open_to_follows: bool,
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
            timezone: String::new(),
            autoplay: true,
            preload_count: default_preload(),
            crossfade_seconds: 0,
            eq: EqConfig::default(),
            eq_presets: Vec::new(),
            profile_visibility: default_audience_private(),
            followers_visibility: default_audience_friends(),
            following_visibility: default_audience_friends(),
            playlists_visibility: default_audience_private(),
            followed_artists_visibility: default_audience_friends(),
            open_to_follows: true,
        }
    }
}

fn default_preload() -> u32 {
    2
}

fn default_audience_friends() -> Audience {
    Audience::Friends
}

fn default_audience_private() -> Audience {
    Audience::Private
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
    // Not a colour: the sentinel that defers to whatever the operator set for this deployment, and
    // to the app's own accent when they set nothing. A new account inherits the site it joined
    // rather than a hardcoded pink, and keeps following if the operator changes it.
    "default".to_string()
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
