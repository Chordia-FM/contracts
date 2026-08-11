//! Backups: what exists, whether it is intact, and what restoring one would cost.
//!
//! ## The shape of this surface, and why
//!
//! A restore is the one irreversible action in the admin panel, so the types here are built around
//! separating **looking** from **doing**. [`BackupVerdict`] is what you get from inspecting an
//! archive without touching the database, and the field that matters most on it is
//! [`BackupVerdict::would_resurrect`]: how many deleted accounts this archive would bring back to
//! life. Reading that number before the restore, rather than discovering it afterwards, is the whole
//! reason verify is a separate call.
//!
//! ## Why there is no "download" shape
//!
//! An archive holds every password hash and TOTP secret on the instance. Serving one through the
//! admin API would put that behind a session cookie, which is a much weaker gate than the private
//! key currently required to read it. Backups are fetched from the host or from object storage.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::EpochMillis;

/// One archive on disk, as listed. Nothing here requires decrypting it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BackupFile {
    /// File name, which is also the id used to verify or restore it. Never a path: the directory is
    /// server-side configuration and a client that could name one could name `/etc/passwd`.
    pub name: String,
    pub size_bytes: i64,
    /// From the filename stamp when it parses, falling back to the file's mtime.
    pub created_at_ms: EpochMillis,
    /// `.age` suffix. An unencrypted archive is a finding, not a feature: it means every credential
    /// on the instance is sitting in a file with only filesystem permissions in front of it.
    pub encrypted: bool,
    /// Present on the Hub's disk, so it can be verified and restored from here.
    pub local: bool,
    /// Present in object storage. **`remote && !local` is the row that matters**: it is the copy
    /// that survived losing the machine, and it is invisible to a listing that reads only one side.
    pub remote: bool,
    /// Size in object storage, when it is there. Separate from `size_bytes` rather than merged so a
    /// disagreement between the two copies is visible instead of averaged away — same name, two
    /// different lengths, means one of them is a truncated upload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_bytes: Option<i64>,
}

/// Whether the erasure ledger is actually being kept, and how current it is.
///
/// Surfaced beside the backup list because the two are one story: an archive without a ledger newer
/// than itself cannot be restored without resurrecting accounts, so a list of backups shown next to
/// a broken ledger is a list of traps.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LedgerStatus {
    /// All four credentials present. False means a restore WILL resurrect deleted accounts.
    pub mirroring: bool,
    /// Object key being written, when mirroring is on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_key: Option<String>,
    /// Rows in `account_erasures` right now.
    pub entries: i64,
    /// The most recent erasure, or `None` if nobody has ever deleted an account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_erasure_ms: Option<EpochMillis>,
}

/// The backups page in one response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BackupIndex {
    /// Newest first.
    pub files: Vec<BackupFile>,
    /// Where the Hub is looking, so "no backups" can be told apart from "looking in the wrong place".
    /// That distinction is the difference between a missing cron job and a missing volume mount.
    pub directory: String,
    /// `None` when no directory is configured at all, which is its own answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<LedgerStatus>,
    /// `None` when no read credentials are configured, so the list is the server's copies alone.
    /// Distinct from a configured remote that returned nothing, which is [`RemoteStatus::reachable`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteStatus>,
}

/// What object storage holds, when the Hub can see it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RemoteStatus {
    /// The listing succeeded. False means the credentials or the bucket are wrong, and the local
    /// list is still correct — a remote that cannot be read must not make the whole tab an error.
    pub reachable: bool,
    /// Why not, when unreachable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    /// Archives seen (the erasure ledger and anything else in the bucket are excluded).
    pub archives: i64,
    /// Their total size. Current versions only; superseded ones are not counted.
    pub total_bytes: i64,
}

/// What inspecting an archive found. Produced without writing anything.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BackupVerdict {
    pub name: String,
    /// The archive decrypted, unpacked, and carries pg_dump's completion marker.
    pub intact: bool,
    /// Why not, when `intact` is false. A wrong key and a truncated file are different problems.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    /// Uncompressed size of the SQL, so a suspiciously small archive is visible before it is applied.
    pub sql_bytes: i64,
    /// **The number that decides whether to proceed.** Accounts named by the CURRENT erasure ledger
    /// that this archive would bring back. `erasures::replay` re-erases them, so this is not a
    /// blocker — but restoring without knowing it is how a GDPR breach happens quietly.
    pub would_resurrect: i64,
    /// Ids of those accounts, capped for display. Full count is `would_resurrect`.
    #[serde(default)]
    pub sample: Vec<Uuid>,
}

/// What a completed restore did.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RestoreReport {
    pub name: String,
    /// Name of the automatic pre-restore dump, so the previous state is recoverable. Written before
    /// anything is dropped; if this is `None` the restore did not proceed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safety_dump: Option<String>,
    /// Accounts re-erased by the ledger replay after the data went in.
    pub re_erased: i64,
    /// Ledger rows asserted afterwards, so the NEXT restore still knows about all of them.
    pub ledger_entries: i64,
    pub duration_ms: i64,
}
