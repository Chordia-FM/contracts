//! Admin surfaces: the moderation queue, the user roster, and the audit log.
//!
//! These were hand-written TypeScript interfaces on the client and bare `serde::Serialize` structs
//! on the server, so the two could disagree and nothing would say so. They are generated from here
//! now, like every other wire shape.
//!
//! Only the shapes an endpoint actually serves live here. The Overview and System payloads
//! (`AdminOverview`, `AdminSystemHealth` and friends) arrive with the endpoints that compute them —
//! a contract for a route that does not exist is not a placeholder, it is a promise nothing keeps.

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
