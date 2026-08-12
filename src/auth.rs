//! Authentication & authorization contracts: account tokens and the capability tokens that
//! gate every audio stream.

use serde::{Deserialize, Serialize};

use crate::{library::PermissionLevel, EpochMillis, Uuid};

/// Claims carried by a user **access token** (short-lived JWT, signed by the Hub).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccessClaims {
    /// Subject - the global user id.
    pub sub: Uuid,
    /// Handle at issue time (convenience; authoritative value lives in the directory).
    pub handle: String,
    /// Issued-at (epoch millis).
    pub iat: EpochMillis,
    /// Expiry (epoch millis).
    pub exp: EpochMillis,
    /// Session id this token belongs to (ties the token to a device session so it can be listed /
    /// revoked). Optional for backward compatibility with tokens minted before sessions existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sid: Option<Uuid>,
    /// Key id used to sign - lets the verifier select the right JWKS key during rotation.
    pub kid: String,
}

/// One active login session ("device"), as shown in account settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SessionInfo {
    pub session_id: Uuid,
    /// When the session began (epoch millis).
    pub created_at: EpochMillis,
    /// Last time the session refreshed (epoch millis).
    pub last_used_at: EpochMillis,
    /// Raw User-Agent captured at last refresh, for a human-readable device label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// True for the session making this request.
    pub current: bool,
}

/// Read-only account overview for the settings Account/Security tabs (`GET /v1/me/account`).
/// Deliberately separate from `UserProfile`, which is hand-built in several places and returned
/// from several endpoints — widening it has a far larger blast radius than a dedicated DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AccountInfo {
    /// The account's email address. `None` for accounts created without one (e.g. OAuth-only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub email_verified: bool,
    /// Whether a password credential exists, so the UI offers "change" vs. "set" a password.
    pub has_password: bool,
    pub discord_linked: bool,
    pub totp_enabled: bool,
    /// Number of registered passkeys. **Always 0 until passkeys ship**; it exists now so the
    /// Security tab's shape does not change if they ever do.
    #[serde(default)]
    pub passkey_count: u32,
}

/// Body of the password-change endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    /// Sign every other device out after the change. Defaults to true — the safe reading of a
    /// password change is that the old one may be compromised.
    #[serde(default = "default_true")]
    pub revoke_other_sessions: bool,
}

fn default_true() -> bool {
    true
}

/// A credential request (email + password) against the Hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    /// When set, the Hub issues a long-lived ("remember me") refresh token.
    #[serde(default)]
    pub remember: bool,
}

/// Issued token pair. The refresh token is opaque (server-side rotation); the access token is a
/// JWT verifiable against the Hub JWKS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: EpochMillis,
}

/// What a capability token authorizes the holder to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum CapabilityAction {
    /// Read/stream an audio resource directly (client → library).
    StreamRead,
    /// Pull a resource for relay (listener's library → DJ's library).
    RelayPull,
    /// Manage acquisition on a library the caller OWNS: interactive Prowlarr search + qBittorrent
    /// grab, directly against the library (the Hub never sees indexer secrets or torrents). Minted
    /// only for library owners.
    ManageAcquisition,
    /// Recover the library server's management token on a device the owner is signed in on (add /
    /// remove folders, rescan). Owner-only; the library returns its management token in exchange, so a
    /// new device manages folders without re-running the one-time pairing flow. Minted only for owners.
    RecoverManagement,
}

/// The resource a capability token is scoped to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum ResourceRef {
    Track { track_id: Uuid },
    Album { album_id: Uuid },
    Library { library_id: Uuid },
}

/// Claims inside a **capability token** - the linchpin of the data-plane security model.
///
/// Minted by the Hub only after verifying friendship + share permissions, then validated
/// **offline** by the target library against the Hub JWKS.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CapabilityClaims {
    /// Subject - the user (or, for relay, the requesting library's owner) being authorized.
    pub sub: Uuid,
    /// Audience - the `server_id` of the library expected to honor this token.
    pub aud: Uuid,
    /// Library the resource lives in.
    pub library_id: Uuid,
    /// What may be accessed.
    pub resource: ResourceRef,
    /// What may be done with it.
    pub action: CapabilityAction,
    /// Permission level granted to the subject for this library (owner → Download, shared → per-share).
    pub permission_level: PermissionLevel,
    /// Room context, set for relay tokens so the DJ's library can scope the grant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<Uuid>,
    /// Unique token id - enables revocation lists and replay detection.
    pub jti: Uuid,
    pub iat: EpochMillis,
    pub exp: EpochMillis,
    pub kid: String,
}

/// Response from register/login: the profile plus a fresh token pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuthResponse {
    pub user: crate::user::UserProfile,
    pub tokens: TokenPair,
}

/// Body for `POST /v1/auth/refresh`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Body for `POST /v1/auth/desktop/authorize` — hand the desktop app the session you already have.
///
/// The desktop app cannot see the website's session: different origin, different storage. So rather
/// than asking someone to sign in a second time, it opens the Hub's own web frontend, where they
/// are usually already signed in, and that page calls this to mint a one-time code for the account
/// it is signed in as. The code goes back over the `chordia://` deep link and is redeemed by
/// [`DesktopExchangeRequest`].
///
/// This is why the desktop app needs no sign-in method of its own. Whatever the website supports —
/// password, Discord, a second factor, something added later — is what the desktop app supports,
/// without knowing anything about any of it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DesktopAuthorizeRequest {
    /// SHA-256 (lowercase hex) of the verifier the desktop app is holding.
    pub challenge: String,
}

/// The one-time code to hand back over the deep link. Worthless without the verifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DesktopAuthorizeResponse {
    pub code: String,
}

/// Body for `POST /v1/auth/desktop/exchange` — the last step of desktop sign-in.
///
/// The desktop app cannot receive the web flow's redirect, so its OAuth round-trip ends at a
/// `chordia://auth/callback` deep link. **Tokens must never travel in that URI**: any other program
/// on the machine can register the same custom scheme, and a session handed to the wrong one is the
/// whole account. So the deep link carries a code that is single-use, expires in a minute, and is
/// worthless on its own — this request trades it for real tokens.
///
/// `verifier` is what makes "worthless on its own" true. The app invents a high-entropy string
/// before opening the browser and sends only its SHA-256 to the Hub; the code is bound to that
/// hash. A rogue app that intercepts the deep link has the code but never saw the verifier, so the
/// exchange fails. (This is PKCE, applied to Chordia's own exchange rather than Discord's.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DesktopExchangeRequest {
    /// The `code` query parameter from the `chordia://auth/callback` deep link.
    pub code: String,
    /// The original secret whose SHA-256 was sent as `challenge` when the flow started.
    pub verifier: String,
}
