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
