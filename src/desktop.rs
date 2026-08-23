//! The desktop app's current release, as the public download page needs it.
//!
//! The installers live on a GitHub release. The page cannot read that release itself: GitHub serves
//! release assets with no `Access-Control-Allow-Origin`, so a browser `fetch` from chordia.dev is
//! blocked, and the filenames carry the version (`Chordia_0.1.5_x64-setup.exe`) so they cannot be
//! hard-coded either without going stale on every release.
//!
//! So the Hub reads it once and republishes it. The URLs handed out are still GitHub's — a 150 MB
//! AppImage has no business travelling through our nginx — and only the small JSON that names them
//! is proxied. Following a link is a navigation rather than a fetch, so CORS never enters into it.

use serde::{Deserialize, Serialize};

use crate::EpochMillis;

/// Which installable artifact this is.
///
/// Deliberately more specific than an operating system: Linux ships two formats and a person on
/// Debian wants the `.deb`, not a choice between two things called "Linux".
///
/// `AndroidApk` is here rather than in a type of its own because the mobile release feed
/// (`GET /v1/mobile/latest`) answers with the same [`DesktopRelease`] shape — a version, notes, and
/// a list of files to download. Two parallel definitions of that would have to be kept in step by
/// hand in Rust, TypeScript and Dart, for no difference any consumer could act on. The three
/// Android builds on a release (universal, arm64-v8a, armeabi-v7a) all carry this variant and are
/// told apart by [`DesktopDownload::filename`]; the feed orders them so the universal APK — the one
/// that installs on any device — is first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum DesktopPlatform {
    Windows,
    LinuxAppImage,
    LinuxDeb,
    MacOs,
    AndroidApk,
}

/// One installer on the current release.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DesktopDownload {
    pub platform: DesktopPlatform,
    /// The asset's own name, shown so somebody can check what they downloaded against the checksums.
    pub filename: String,
    /// An absolute URL on the release host. Not proxied.
    pub url: String,
    pub size_bytes: u64,
}

/// What the download page renders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DesktopRelease {
    /// `0.1.5`, without a leading `v`. The same string the updater compares against.
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<EpochMillis>,
    /// The release page, for anyone who wants the notes or an asset this list does not name.
    pub notes_url: String,
    /// `SHA256SUMS`, when the release carries one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksums_url: Option<String>,
    /// Empty when the release host could not be reached. The page says so rather than rendering an
    /// empty frame that reads as "there is no desktop app".
    #[serde(default)]
    pub downloads: Vec<DesktopDownload>,
}
