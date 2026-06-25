//! Library & selective-sharing contracts. A user's self-hosted server may host several logical
//! **libraries** (named collections), each shared independently.

use serde::{Deserialize, Serialize};

use crate::{user::PublicUser, EpochMillis, Uuid};

/// What a grantee may do with a shared library. Read-only for the MVP; `Download` is reserved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum PermissionLevel {
    /// Stream only.
    Read,
    /// Stream + persist an own-copy (reserved, post-MVP).
    Download,
}

/// Summary of a logical library as registered with the Hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibrarySummary {
    pub id: Uuid,
    pub owner_id: Uuid,
    /// Display name, e.g. "Hi-Fi Archive", "Family Collection".
    pub name: String,
    /// The self-hosted server (`server_id`) that physically hosts this library.
    pub server_id: Uuid,
    /// Optional Phosphor icon name chosen by the owner; None → default icon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub track_count: u32,
    pub created_at: EpochMillis,
}

/// A share grant: which user may access which library, at what level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LibraryShare {
    pub library_id: Uuid,
    pub grantee: PublicUser,
    pub permission_level: PermissionLevel,
    pub created_at: EpochMillis,
}

/// Create or update a share (Hub-side, requires an accepted friendship).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ShareRequest {
    pub library_id: Uuid,
    pub grantee_id: Uuid,
    pub permission_level: PermissionLevel,
}
