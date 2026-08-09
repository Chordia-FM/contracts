//! Admin surfaces: the moderation queue, the user roster, and the audit log.
//!
//! These were hand-written TypeScript interfaces on the client and bare `serde::Serialize` structs
//! on the server, so the two could disagree and nothing would say so. They are generated from here
//! now, like every other wire shape.
//!
//! Every shape here is served by an endpoint. Nothing is declared ahead of the route that computes
//! it — a contract for a route that does not exist is not a placeholder, it is a promise nothing
//! keeps.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::EpochMillis;

/// One side of an edit: the fields that moved, as display strings.
///
/// Deliberately flat and stringly-typed rather than `serde_json::Value`. Three reasons, in order of
/// weight: `serde_json` is a DEV dependency of this crate on purpose, so every consumer stays lean;
/// ts-rs maps `Value` to `any`, which this project does not allow; and a diff is rendered as text
/// anyway, so the fidelity a `Value` would preserve is fidelity nothing reads.
///
/// `None` means the field was unset — distinct from `Some("")`, which is a field explicitly cleared.
pub type AuditDiff = BTreeMap<String, Option<String>>;

/// One account in the admin roster.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminUserRow {
    pub id: Uuid,
    pub handle: String,
    pub email: String,
    pub display_name: String,
    pub created_at_ms: EpochMillis,
    pub suspended: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suspended_reason: Option<String>,
}

/// One user report awaiting (or past) moderation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReportRow {
    pub id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reporter_handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_handle: Option<String>,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub status: String,
    pub created_at_ms: EpochMillis,
}

/// One entry in the append-only privileged-action log.
///
/// `target_type` and `target_label` are what make an entry legible. A bare `target_id` is a UUID
/// with no way to resolve it — worse, `delete_user` destroys the row it points at, so after the fact
/// there is nothing left to resolve it *against*. The label is therefore captured at write time and
/// denormalized here on purpose; it is a record of what the target was called when the thing
/// happened, not a live lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuditEntry {
    /// BIGSERIAL, strictly monotonic — which is what keyset pagination pages on.
    pub id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_handle: Option<String>,
    pub action: String,
    /// `admin` | `moderation` | `catalog` | `auth` | `system`. Coarse enough to filter by.
    pub category: String,
    /// `user` | `artist` | `album` | `track` | `label` | `report` | `suggestion`, or absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// The entity's relevant fields before and after the change, narrowed to what actually moved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<AuditDiff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<AuditDiff>,
    /// True when `target_type`/`target_label` were INFERRED by the migration rather than recorded.
    ///
    /// Surfaced rather than hidden: a backfilled label is the target's name *today*, not its name
    /// when the action happened, and for a deleted target there is no label at all. An operator
    /// reading an old entry needs to know which kind they are looking at.
    pub backfilled: bool,
    pub created_at_ms: EpochMillis,
}

/// A page of the audit log, plus the cursor for the page after it.
///
/// Keyset, not offset: the log is append-only and unbounded, so an offset page shifts under the
/// reader every time anything privileged happens while they are looking at it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuditPage {
    pub rows: Vec<AuditEntry>,
    /// Pass back as `before_id` for the next page. `None` means this was the last one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_before_id: Option<i64>,
}

/// The distinct values actually present in the log, for populating the filter dropdowns.
///
/// Derived from the data rather than from a hard-coded list, so an action added on the server shows
/// up in the filter without a matching client release — and one that has never fired does not
/// offer an empty filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuditFacets {
    pub actions: Vec<String>,
    pub categories: Vec<String>,
    pub actors: Vec<AuditActor>,
}

/// An account that appears as an actor in the log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AuditActor {
    pub id: Uuid,
    pub handle: String,
}

// ── Overview ──────────────────────────────────────────────────────────────────────────────────

/// One day of hub-wide activity.
///
/// `listeners` is what separates a busy day from one heavy listener, and it cannot be derived from
/// `plays` — which is exactly why it is a column here rather than something the client computes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminDayPoint {
    /// Local calendar day, `YYYY-MM-DD`. A DATE read as text: this crate's sqlx has no chrono.
    pub day: String,
    pub plays: i64,
    pub listeners: i64,
    pub ms_played: i64,
}

/// One day of signups, for the growth chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminSignupPoint {
    pub day: String,
    pub signups: i64,
}

/// A hub-wide top entity, with enough to render it as more than a name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminTopEntity {
    pub id: Uuid,
    pub name: String,
    /// Artist for an album/track, absent for an artist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_hash: Option<String>,
    pub plays: i64,
}

/// Who is on the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminPeople {
    pub total: i64,
    pub new_7d: i64,
    pub new_30d: i64,
    pub suspended: i64,
    pub admins: i64,
    pub verified: i64,
    pub with_totp: i64,
}

/// What they are doing.
///
/// `dau`/`wau`/`mau` are distinct-listener counts over the trailing 1/7/30 days, read from the daily
/// rollup — never from `listening_events`, which is the partitioned fact table and the one thing an
/// admin refresh must not scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminActivity {
    pub plays_today: i64,
    pub plays_7d: i64,
    pub plays_30d: i64,
    pub ms_played_30d: i64,
    pub dau: i64,
    pub wau: i64,
    pub mau: i64,
}

/// What is in the catalog, and how much of the fleet is reachable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminCatalog {
    pub artists: i64,
    pub albums: i64,
    pub tracks: i64,
    pub labels: i64,
    pub playlists: i64,
    pub libraries: i64,
    pub servers_online: i64,
    pub servers_total: i64,
}

/// The Overview tab's whole payload, in one request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminOverview {
    pub people: AdminPeople,
    pub activity: AdminActivity,
    pub catalog: AdminCatalog,
    pub plays_series: Vec<AdminDayPoint>,
    pub signups_series: Vec<AdminSignupPoint>,
    pub top_artists: Vec<AdminTopEntity>,
    pub top_albums: Vec<AdminTopEntity>,
    pub top_tracks: Vec<AdminTopEntity>,
}

// ── System health ─────────────────────────────────────────────────────────────────────────────

/// One background rollup and how far behind it is.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminRollupStat {
    pub name: String,
    /// How far behind the newest event this rollup has processed, in seconds.
    pub lag_seconds: i64,
    pub last_event_ms: EpochMillis,
}

/// A count of download jobs in one status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminQueueStat {
    pub status: String,
    pub count: i64,
}

/// A table and what it costs on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminTableSize {
    pub name: String,
    pub bytes: i64,
    pub rows: i64,
}

/// How much of the catalog is still waiting on an enrichment worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminEnrichmentBacklog {
    pub artists_missing_art: i64,
    pub artists_never_enriched: i64,
    pub tracks_missing_recording: i64,
    pub tracks_missing_isrcs: i64,
    pub tracks_missing_lyrics: i64,
}

/// Operator health, everything an admin would otherwise reach for psql to answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminSystemHealth {
    pub version: String,
    pub migrations_applied: i64,
    pub db_bytes: i64,
    pub listening_events_partitions: i64,
    pub rollups: Vec<AdminRollupStat>,
    pub queue: Vec<AdminQueueStat>,
    pub biggest_tables: Vec<AdminTableSize>,
    pub enrichment: AdminEnrichmentBacklog,
}

// ── Users ─────────────────────────────────────────────────────────────────────────────────────

/// One row of the admin roster, with everything the table shows.
///
/// Separate from [`AdminUserRow`] rather than an extension of it: that shape is also what the
/// suspend/delete flows round-trip, and widening it would make every one of those carry seven
/// columns they do not use.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminUserDetail {
    pub id: Uuid,
    pub handle: String,
    pub email: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub created_at_ms: EpochMillis,
    pub suspended: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suspended_reason: Option<String>,
    pub email_verified: bool,
    pub totp_enabled: bool,
    pub is_admin: bool,
    pub libraries: i64,
    pub plays: i64,
    /// Last time any of this account's sessions was used. Absent for an account that has never
    /// signed in since sessions were tracked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_ms: Option<EpochMillis>,
}

/// A page of the roster, plus the total the filters match.
///
/// Offset paging here, not keyset: the roster is small, sortable by any column, and an admin
/// genuinely wants "page 3 of 7" — the properties that made keyset right for the audit log are all
/// absent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AdminUserPage {
    pub rows: Vec<AdminUserDetail>,
    pub total: i64,
}
