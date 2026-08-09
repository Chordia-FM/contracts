//! The read-only data explorer: browse, sort and filter the Hub's own tables.
//!
//! **Read-only by construction, not by convention.** There is no write endpoint, no `UPDATE` path,
//! and the set of readable tables is a hard-coded allow-list rather than anything derived from
//! `information_schema` — a table added to the database is not browsable until someone decides it
//! should be. That is what keeps `users.password_hash` and `totp_secret` out of a browser.

use serde::{Deserialize, Serialize};

/// How a column should be rendered, decided server-side from what it actually holds.
///
/// The client cannot infer this: a `TEXT` column holding a sha256 and one holding a display name
/// are the same type on the wire, and only the table definition knows which is which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ColumnKind {
    Text,
    Number,
    Bool,
    /// Epoch milliseconds. Rendered as a date, and sortable as a number.
    Timestamp,
    Uuid,
    /// A sha256 that resolves through `/v1/images/{hash}` — rendered as a thumbnail.
    Image,
    /// A JSON blob. Shown collapsed; the row detail expands it.
    Json,
    /// A binary column. The SIZE travels, never the payload.
    ///
    /// This exists because of a live incident: `images.bytes` is the image itself, it was declared
    /// `Number`, and `bytes::text` shipped the hex encoding of every image on the page — tens of
    /// megabytes per request, and `Number("\\x89504e...")` is `NaN`. A binary column has no
    /// display form, so the only honest thing to send is how big it is.
    Bytes,
}

/// One column of a browsable table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExplorerColumn {
    pub name: String,
    pub kind: ColumnKind,
    /// Whether `ORDER BY` accepts this column. Sorting an unindexed text column on a large table is
    /// a sequential scan an operator did not ask for.
    pub sortable: bool,
    /// Whether the free-text filter searches this column.
    pub searchable: bool,
}

/// A table the explorer will show, and what it looks like.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExplorerTable {
    /// The key the API takes. Never a raw table name from the caller.
    pub key: String,
    pub label: String,
    /// Roughly how many rows, from the planner's statistics rather than a count.
    pub approx_rows: i64,
    pub columns: Vec<ExplorerColumn>,
    /// The column whose value titles a row in the detail view.
    pub title_column: String,
    /// The image column, when the table has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_column: Option<String>,
    /// What the table costs on disk, indexes and TOAST included.
    #[serde(default)]
    pub size_bytes: i64,
}

/// One row, as display values.
///
/// Everything is a string (or absent for NULL) rather than a typed union: the explorer renders
/// values, it does not compute with them, and a `Vec<serde_json::Value>` would put `any` in the
/// TypeScript for no gain. `ColumnKind` already says how to draw each one.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExplorerRow {
    /// Primary key, for the detail view and for React keys.
    pub id: String,
    pub cells: Vec<Option<String>>,
}

/// One page of a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ExplorerPage {
    pub table: ExplorerTable,
    pub rows: Vec<ExplorerRow>,
    /// Rows matching the filter, counted up to a cap.
    ///
    /// Exact below the cap, because an operator paging to the end of a wrong estimate is worse than
    /// the count. Capped above it, because an exact `count(*)` behind every keystroke of a filter is
    /// a sequential scan the explorer has no business asking for.
    pub total: i64,
    /// `total` hit the cap and the real figure is higher. Rendered as `10,000+`.
    #[serde(default)]
    pub total_capped: bool,
}
