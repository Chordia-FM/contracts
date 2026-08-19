//! Insights / "Wrapped" contracts. The output of the centralized analytics engine.

use serde::{Deserialize, Serialize};

use crate::user::PublicUser;
use crate::{EpochMillis, Uuid};

/// Aggregation window for a stats report. Mirrors the Last.fm-style windows: 7-day, 1/3/6/12
/// month, and all-time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum Period {
    /// The current local day plus the one before it — "what have I been listening to today".
    /// Bucketed hourly rather than daily, since a one-day window has only one daily bucket.
    Day,
    /// Trailing 7 days.
    Week,
    /// Trailing 30 days (~1 month).
    Month,
    /// Trailing 90 days (~3 months).
    Quarter,
    /// Trailing 180 days (~6 months).
    HalfYear,
    /// Trailing 365 days (~12 months).
    Year,
    /// All recorded history.
    Overall,
    /// An explicit `[from, to)` range chosen by the caller (Deep Analytics). Only ever *produced*,
    /// as the echoed `period` of a report that was requested with both `from` and `to` bounds —
    /// `skip_deserializing` means no query can name it directly, so period-driven windows
    /// (playlist stats, movers, discovery) can never receive it by accident. The report's real
    /// bounds are its `window_start`/`window_end`.
    #[serde(skip_deserializing)]
    Custom,
}

/// A figure measured against the equivalent preceding window, so a number can be read as a trend
/// rather than in isolation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Compared {
    pub current: u64,
    pub previous: u64,
    /// Fractional change, for example `-0.73` for a 73% drop. `None` when `previous` is zero,
    /// because percentage change from nothing is undefined; clients should describe this as
    /// "new" rather than rendering infinity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<f32>,
}

/// Grain the over-time series is bucketed at, so the client can label its x-axis correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum BucketGranularity {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

/// One bucket in a time series of listening activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TimeBucket {
    /// Inclusive start of the bucket (epoch millis, truncated in the requested timezone).
    pub start: EpochMillis,
    pub plays: u32,
    pub ms_played: u64,
}

/// Plays attributed to albums released in one decade.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DecadeBucket {
    /// The decade's first year, for example `1990`.
    pub decade: i16,
    pub plays: u32,
    pub ms_played: u64,
}

/// Exclusive album-genre shares over time. Albums may carry multiple genres; for a stacked chart,
/// each play is assigned once to the highest-ranked top genre on its album. Plays whose album has
/// none of the top genres, including unresolved albums and albums without genres, go to the
/// trailing literal `"other"` entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GenreTrend {
    /// Top album genres in descending whole-window play order, followed by `"other"`.
    pub genres: Vec<String>,
    /// Chronological buckets. Every `plays` vector is parallel to `genres` and sums to all plays in
    /// that time bucket.
    pub buckets: Vec<GenreTrendBucket>,
}

/// One local-calendar bucket of exclusive album-genre play counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct GenreTrendBucket {
    pub start: EpochMillis,
    /// Counts parallel to [`GenreTrend::genres`], including its trailing `"other"` count.
    pub plays: Vec<u32>,
}

/// Five normalized measures of listening behaviour over one local-calendar reporting window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Fingerprint {
    /// `1 - Gini` of daily play counts across every calendar day in the window, including silent
    /// days. `1.0` means every day has the same play count; an empty window is `0.0`.
    pub consistency: f32,
    /// Distinct resolved artists, albums, and tracks first heard in the window divided by all
    /// distinct resolved artists, albums, and tracks played in it. The three entity kinds share
    /// one denominator; an empty denominator is `0.0`.
    pub discovery: f32,
    /// Resolved-track plays after that track's first play in the window divided by all resolved-
    /// track plays in the window. `0.7` means 70% of resolved plays repeated a track already heard
    /// during this window.
    pub replay: f32,
    /// Normalized Herfindahl-Hirschman Index of resolved artist play shares:
    /// `(HHI - 1/n) / (1 - 1/n)`. `0.0` is an even split across the `n` artists and `1.0` is a
    /// single artist; no resolved artists is `0.0`.
    pub concentration: f32,
    /// Population coefficient of variation of daily play counts, including silent days, mapped to
    /// `CV / (1 + CV)`. `0.0` is identical volume every day; values approach `1.0` as day-to-day
    /// volume becomes more volatile. An empty window is `0.0`.
    pub variance: f32,
}

/// A user's listening fingerprint and, when available, the arithmetic mean of fingerprints for
/// Hub users with at least 20 plays in the same window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FingerprintReport {
    pub you: Fingerprint,
    /// Omitted when no Hub user meets the minimum sample size; never synthesized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_average: Option<Fingerprint>,
}

/// Listening intensity by local weekday and hour.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ClockGrid {
    /// IANA timezone used to place events into local calendar cells.
    pub timezone: String,
    /// Exactly 168 row-major entries: `dow * 24 + hour`, with Sunday at row zero.
    pub cells: Vec<u32>,
    /// Highest non-zero play count in any cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak: Option<u32>,
}

/// Chart-oriented listening data for a period: an activity time series plus the listening-clock
/// (hour-of-day) and weekday distributions. Computed live from the partitioned fact table, scoped
/// to one user over a bounded window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ListeningCharts {
    pub period: Period,
    pub window_start: EpochMillis,
    pub window_end: EpochMillis,
    pub granularity: BucketGranularity,
    /// IANA timezone used for the local calendar window and time buckets.
    pub timezone: String,
    /// Plays bucketed across the window, chronological. Daily for windows up to a year; monthly for
    /// `Overall`. Doubles as a calendar-heatmap source at day granularity.
    pub over_time: Vec<TimeBucket>,
    /// Plays by local hour-of-day, exactly 24 entries (index = hour 0 to 23).
    pub clock: Vec<u32>,
    /// Plays by local day-of-week, exactly 7 entries (index 0 = Sunday through 6 = Saturday).
    pub weekday: Vec<u32>,
    pub clock_grid: ClockGrid,
}

/// One entry in the full scrobble history feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HistoryEntry {
    /// The play event's idempotency id (also the keyset-pagination tiebreak).
    pub event_id: Uuid,
    /// Resolved catalog track id, if the play matched the catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<Uuid>,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub played_at: EpochMillis,
    pub ms_played: u64,
}

/// A page of scrobble history, newest first, with a keyset cursor for the next page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HistoryPage {
    pub entries: Vec<HistoryEntry>,
    /// Pass these back as `before_ms` / `before_id` to fetch the next page. Both `None` at the end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_before_ms: Option<EpochMillis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_before_id: Option<Uuid>,
}

/// One ranked entry in a top-N list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TopItem {
    /// Catalog id of the artist/track/album.
    pub id: Uuid,
    pub name: String,
    pub plays: u32,
    pub ms_played: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// One month in the growth of the caller's liked songs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LikedPoint {
    /// First day of the month, `YYYY-MM-DD`.
    pub month: String,
    /// Liked tracks held at the END of that month.
    ///
    /// Counted from the likes that are STILL IN PLACE, because an unlike leaves no record: the row
    /// is deleted, so a song liked in March and unliked in April was never here as far as this can
    /// tell. The line therefore reads "how your current collection accumulated", not "how many
    /// songs you had liked at the time", and those are different questions with the same shape.
    pub total: u32,
}

/// One artist's share of the caller's liked songs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LikedArtist {
    pub id: Uuid,
    pub name: String,
    /// Liked tracks credited to this artist. Deliberately not a play count — this list answers
    /// "whose songs did you keep", which is a different ranking from "whose songs did you play".
    pub liked: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// The caller's liked songs in numbers, for the disclosure on the Liked Songs page.
///
/// Everything derived from listening honours the caller's retention floor, so the ratios below
/// describe the window their insights already show rather than quietly reaching past it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LikedStats {
    /// Liked tracks right now.
    pub total: u32,
    /// Their combined length. `0` when none of them resolve to a catalog track with a duration.
    pub total_duration_ms: u64,
    /// Oldest and newest like still in place.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_liked: Option<EpochMillis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_liked: Option<EpochMillis>,
    /// Cumulative growth, one point per month from the first like to now.
    pub history: Vec<LikedPoint>,
    /// Distinct tracks the caller has played inside the retention window.
    pub played_tracks: u32,
    /// How many of those they went on to like. With `played_tracks` at 0 the ratio is undefined,
    /// which is why both numbers are sent rather than a percentage computed here.
    pub played_liked: u32,
    /// Liked tracks with no play on record in the window. "Saved for later" made countable.
    pub never_played: u32,
    /// The caller's plays across their liked tracks.
    pub liked_plays: u64,
    /// The caller's most-played tracks that they have NOT liked — the songs the heart missed.
    pub unliked_favourites: Vec<TopItem>,
    /// Artists best represented in the liked list.
    pub top_artists: Vec<LikedArtist>,
}

/// One consecutive run of active listening days.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Streak {
    /// First local calendar day in the run (`YYYY-MM-DD`).
    pub start_day: String,
    /// Last local calendar day in the run (`YYYY-MM-DD`).
    pub end_day: String,
    pub days: u32,
    pub plays: u32,
    /// True when the run reaches today or yesterday in the requested timezone.
    pub active: bool,
}

/// A continuous listening session, split after a 30-minute idle gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ListeningSession {
    pub started_at: EpochMillis,
    pub ended_at: EpochMillis,
    pub tracks: u32,
    pub ms_played: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_artist: Option<String>,
}

/// A notable ordinal play in a user's listening history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Milestone {
    pub ordinal: u64,
    pub played_at: EpochMillis,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// Personal listening records and all-time milestones for one reporting window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ListeningRecords {
    pub period: Period,
    pub window_start: EpochMillis,
    pub window_end: EpochMillis,
    pub timezone: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longest_streak: Option<Streak>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_streak: Option<Streak>,
    pub active_days: u32,
    /// Mean plays per calendar day in the window, including silent days.
    pub avg_plays_per_day: f32,
    /// Current and previous averages rounded to the nearest whole play per day. The precise current
    /// value remains available in `avg_plays_per_day`.
    pub avg_plays_per_day_compared: Compared,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biggest_day: Option<TimeBucket>,
    /// Play count of the busiest local-calendar day in each window (zero when a window is empty).
    pub biggest_day_compared: Compared,
    /// Longest sessions first, capped at ten.
    pub top_sessions: Vec<ListeningSession>,
    /// Round-number play milestones, ascending.
    pub milestones: Vec<Milestone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_scrobble: Option<Milestone>,
    /// True when the day-grained figures could not be computed yet: the UTC report is served from
    /// the daily rollup, and on a Hub whose aggregator has never completed a pass there is no
    /// rollup to read. The streaks, `active_days`, the averages and `biggest_day` above are then
    /// placeholders rather than facts, and a client should say "still being computed" instead of
    /// rendering a zero-day history as if it were real. Sessions and milestones are unaffected
    /// (they scan the fact table directly), as are non-UTC reports.
    #[serde(default)]
    pub day_stats_pending: bool,
}

/// How many distinct catalog entities were played and newly discovered in a reporting window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DiscoveryStats {
    pub period: Period,
    pub window_start: EpochMillis,
    pub window_end: EpochMillis,
    pub artists_played: u32,
    pub artists_played_compared: Compared,
    pub artists_new: u32,
    pub albums_played: u32,
    pub albums_played_compared: Compared,
    pub albums_new: u32,
    pub tracks_played: u32,
    pub tracks_played_compared: Compared,
    pub tracks_new: u32,
    /// Share of window plays whose track was heard before the window (`0.0..=1.0`).
    pub repeat_rate: f32,
    pub top_new_artists: Vec<TopItem>,
}

/// One ranked catalog entry in a current-versus-previous-window chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChartEntry {
    pub id: Uuid,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub plays: u32,
    pub ms_played: u64,
    pub rank: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_rank: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_plays: Option<u64>,
}

/// One page of a listener's full ranked library. Offset-paged rather than keyset: the ranking is
/// dense and recomputed per window, so there is no stable cursor to page from, and a reader jumping
/// to "page 7" expects ranks 121–140 rather than wherever a cursor happened to land.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChartPage {
    pub kind: EntityKind,
    pub period: Period,
    pub window_start: EpochMillis,
    pub window_end: EpochMillis,
    /// Distinct entities of this kind the listener played in the window — the denominator behind
    /// "#7 of 412", and what tells the client whether another page exists.
    pub total: u32,
    pub offset: u32,
    pub entries: Vec<ChartEntry>,
}

/// A ranked item whose position changed between adjacent reporting windows.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RankMove {
    pub item: ChartEntry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<i32>,
}

/// Current-window chart movement compared with the immediately preceding window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RankMovers {
    pub kind: EntityKind,
    pub period: Period,
    pub climbers: Vec<RankMove>,
    pub fallers: Vec<RankMove>,
    pub newcomers: Vec<ChartEntry>,
}

/// One past year's plays on today's local month and day.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OnThisDayYear {
    pub year: u16,
    pub plays: u32,
    /// Newest-first sample of that day's plays.
    pub entries: Vec<HistoryEntry>,
}

/// Listening-history memories whose local date matches today's month and day.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct OnThisDay {
    pub timezone: String,
    /// Most recent year first.
    pub years: Vec<OnThisDayYear>,
}

/// A listening-insights report for a user over a period.
///
/// Computed live from the partitioned `listening_events` fact table, scoped to one user over a
/// bounded window. (This previously claimed to be served from precomputed rollups; it never was —
/// the fixed-grain rollups cannot answer an arbitrary rolling window precisely.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct WrappedReport {
    pub user_id: Uuid,
    pub period: Period,
    /// Inclusive start of the window (epoch millis).
    pub window_start: EpochMillis,
    /// Exclusive end of the window (epoch millis).
    pub window_end: EpochMillis,
    /// IANA timezone used to align the current and preceding windows to local midnights.
    pub timezone: String,
    pub total_plays: u32,
    pub total_plays_compared: Compared,
    pub total_ms_played: u64,
    pub total_ms_played_compared: Compared,
    pub unique_tracks: u32,
    pub unique_tracks_compared: Compared,
    pub unique_artists: u32,
    pub unique_artists_compared: Compared,
    pub unique_albums: u32,
    pub unique_albums_compared: Compared,
    pub top_artists: Vec<TopItem>,
    pub top_tracks: Vec<TopItem>,
    pub top_albums: Vec<TopItem>,
    /// Top genres by play count. `id` is a stable hash of the genre name (genres aren't catalog
    /// entities), and `image_url` is always `None`.
    pub top_genres: Vec<TopItem>,
    /// Continuous decade axis from the earliest dated album played through the current decade.
    pub decades: Vec<DecadeBucket>,
    /// Share of plays whose album release date cannot be resolved (`0.0..=1.0`).
    pub undated_release_share: f32,
    /// Album genres over the same local-calendar buckets used by listening charts.
    pub genre_trend: GenreTrend,
    pub fingerprint: FingerprintReport,
}

/// Which kind of catalog entity a per-entity stats query is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum EntityKind {
    Artist,
    Album,
    Track,
}
/// Whose listening an entity page is reporting on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum StatsScope {
    /// The requesting listener's own plays.
    #[default]
    Me,
    /// Every listener on the hub, over the same window.
    Global,
}

/// A user's personal listening stats for one catalog entity (artist/album/track). This is the data
/// behind the "your stats" panels on the catalog detail pages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EntityStats {
    pub kind: EntityKind,
    /// Whose plays these figures cover. Echoed back so a page can label itself honestly rather
    /// than trusting the toggle it sent.
    #[serde(default)]
    pub scope: StatsScope,
    pub id: Uuid,
    pub period: Period,
    pub window_start: EpochMillis,
    pub window_end: EpochMillis,
    pub granularity: BucketGranularity,
    pub total_plays: u32,
    pub total_ms_played: u64,
    /// First/last time the user played this entity (epoch millis). `None` if never played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_played: Option<EpochMillis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played: Option<EpochMillis>,
    /// 1-based rank among the user's entities of this kind in the requested period. `None` if the
    /// user has never played it in the period.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    /// Number of the user's played entities included in `rank`. `None` when `rank` is `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_total: Option<u32>,
    /// Hub-wide all-time plays from the entity's global rollup. `None` until the rollup has a row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_plays: Option<u64>,
    /// Local-calendar play trend (chronological), for a sparkline.
    pub trend: Vec<TimeBucket>,
}

/// Page-only listening detail for one catalog entity. Kept separate from [`EntityStats`] so the
/// lightweight catalog-page panel does not wait on clock and top-list scans.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct EntityBreakdown {
    pub kind: EntityKind,
    #[serde(default)]
    pub scope: StatsScope,
    pub id: Uuid,
    pub period: Period,
    pub window_start: EpochMillis,
    pub window_end: EpochMillis,
    /// When the caller listens to this entity, in their own timezone.
    pub clock: ClockGrid,
    /// The caller's most-played tracks of this entity: an album's or artist's tracks; empty when
    /// the entity is a track.
    pub top_tracks: Vec<TopItem>,
    /// An artist's albums by the caller's play count. Empty for album and track.
    pub top_albums: Vec<TopItem>,
}

/// One entry in the friends' recent-activity feed, a play by a friend whose privacy allows it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct FriendScrobble {
    pub user_id: Uuid,
    pub handle: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub title: String,
    pub artist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub played_at: EpochMillis,
}

/// Taste-compatibility between the caller and another user: a 0 to 1 overlap score plus the artists
/// they share. Only returned when the other user's privacy permits it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Compatibility {
    pub user_id: Uuid,
    pub handle: String,
    pub display_name: String,
    /// Cosine similarity (0.0 to 1.0) over the two users' top-artist play vectors.
    pub score: f32,
    /// Artists both users have played, most-shared first (capped).
    pub shared_artists: Vec<TopItem>,
    /// The detailed breakdown behind the score. Present only when the VIEWER's plan includes
    /// `taste_match_summary`; omitted entirely otherwise, so the free shape is byte-identical to
    /// what it was before the field existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<TasteSummary>,
}

/// The paid deep-dive behind a compatibility score: what two listeners share, when they both
/// listen, and what they found at the same time. Every figure is bounded by each side's OWN
/// retention window — a paid viewer buys no reach into a free friend's hidden history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TasteSummary {
    /// Albums both have played, ranked by the smaller of the two play counts (`plays` carries that
    /// shared strength), top 10.
    pub shared_albums: Vec<TopItem>,
    /// Tracks both have played, ranked and capped like `shared_albums`.
    pub shared_tracks: Vec<TopItem>,
    /// Album genres both have played. `id` is a stable hash of the genre name (genres aren't
    /// catalog entities) and `image_url` is always absent.
    pub shared_genres: Vec<TopItem>,
    /// Album-release decades, one entry per decade on a continuous axis from the earliest decade
    /// either side has played through the latest.
    pub decades: Vec<DecadeSplit>,
    /// Histogram intersection (0.0 to 1.0) of the two decade distributions: 1.0 means the same
    /// era shares, 0.0 means no overlap at all. 0.0 when either side has no dated plays.
    pub era_overlap: f32,
    /// Plays by local hour of day over the trailing 90 days, exactly 24 entries (index = hour).
    /// Each side's hours are measured in their own timezone — the comparison is "when in your day
    /// do you each listen", not "are you awake at the same instant".
    pub hours: Vec<HourSplit>,
    /// Histogram intersection (0.0 to 1.0) of the two hour-of-day distributions.
    pub time_overlap: f32,
    /// Artists whose first listen (week-grained) happened in the same week for both sides, with at
    /// least 5 plays each, strongest shared interest first, top 5.
    pub discovered_together: Vec<DiscoveredTogether>,
}

/// One decade's plays, yours against theirs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DecadeSplit {
    /// The decade's first year, for example `1990`.
    pub decade: i32,
    pub you: i64,
    pub them: i64,
}

/// One local hour of day's plays, yours against theirs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct HourSplit {
    /// Local hour of day, 0 to 23.
    pub hour: i32,
    pub you: i64,
    pub them: i64,
}

/// Something both listeners first played in the same week.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct DiscoveredTogether {
    /// Display name of the shared discovery (an artist).
    pub item: String,
    /// The calendar month the shared first week falls in, `YYYY-MM`.
    pub month: String,
}

/// A user's shareable public listening profile. Listening stats are populated only when the viewer
/// is allowed to see them (per the target's scrobble privacy); otherwise `private` is true and the
/// lists are empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PublicProfile {
    pub user: PublicUser,
    /// When the account was created (epoch millis).
    pub created_at: EpochMillis,
    /// True when the viewer may not see this user's **listening activity** — `total_plays`,
    /// `top_artists`, `top_tracks` and `recent` are then empty. Kept under its original name for
    /// compatibility; it says nothing about the profile's other surfaces, which carry their own
    /// visibility signals below.
    pub private: bool,
    /// **The whole profile is withheld** — the viewer may not see this account at all, and every
    /// field below is a placeholder rather than a fact.
    ///
    /// Distinct from `private`, which walls only the listening activity. The two are different
    /// walls with different copy: conflating them produces a page announcing "this profile is
    /// private" while showing the person's bio, banner and links.
    ///
    /// Explicit rather than inferred. The locked shell is a zeroed DTO, and a genuinely visible
    /// profile can produce a byte-identical one — a new account with default settings, no bio and
    /// no followers, or ANY profile on an instance with the social layer switched off. A client
    /// guessing from field shape therefore walls real profiles, which is precisely the "client
    /// decides visibility" mistake the rest of this DTO is shaped to prevent.
    #[serde(default)]
    pub hidden: bool,
    pub total_plays: u32,
    pub top_artists: Vec<TopItem>,
    pub top_tracks: Vec<TopItem>,
    pub recent: Vec<RecentPlay>,
    /// Free-text profile bio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// Resolved banner image URL (the Hub image endpoint), when one is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    #[serde(default)]
    pub links: Vec<crate::user::ProfileLink>,
    #[serde(default)]
    pub follower_count: u32,
    #[serde(default)]
    pub following_count: u32,
    #[serde(default)]
    pub playlist_count: u32,
    /// The viewer follows this account.
    #[serde(default)]
    pub viewer_follows: bool,
    /// This account follows the viewer.
    #[serde(default)]
    pub follows_viewer: bool,
    /// Friendship edge between viewer and this account, when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub friendship: Option<crate::social::FriendshipStatus>,
    /// Whether the viewer may follow this account (false when already following, when it is their
    /// own profile, or when the target is not open to follows).
    #[serde(default)]
    pub can_follow: bool,
    /// Whether the viewer may open the followers list.
    #[serde(default)]
    pub followers_visible: bool,
    /// Whether the viewer may open the following list.
    #[serde(default)]
    pub following_visible: bool,
    /// The user's playlists. **The `Option` IS the visibility signal**: `None` means the viewer may
    /// not see this surface at all (render nothing), `Some([])` means it is visible and empty
    /// (render the empty state). The client never decides visibility — the server has already
    /// applied `playlists_visibility` and each playlist's own `PlaylistVisibility`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playlists: Option<Vec<crate::catalog::Playlist>>,
    /// The user's followed artists, with the same `None` = not visible / `Some([])` = visible and
    /// empty contract as `playlists`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub followed_artists: Option<Vec<crate::social::ProfileArtist>>,
    /// Badges this account carries. Present on the locked shell too: a badge is identity, like the
    /// handle, and withholding it would make a moderator unrecognisable on exactly the profile where
    /// knowing they are staff matters.
    #[serde(default)]
    pub badges: Vec<crate::billing::ProfileBadge>,
    /// The accent this profile paints itself in for visitors, when the owner is entitled to it and
    /// the viewer has not opted out of seeing other people's accents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<ProfileAccent>,
}

/// A profile's own colour, applied to that page only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProfileAccent {
    /// A CSS colour. Already resolved: a time-varying accent is flattened, because a visitor's page
    /// must not animate.
    pub primary: String,
    /// Two or more stops when the owner chose a gradient; empty otherwise.
    #[serde(default)]
    pub gradient: Vec<String>,
}

/// Listening stats for one playlist, behind `GET /v1/playlists/{id}/stats`.
///
/// Deliberately its own type rather than a widened [`EntityKind`]: a playlist is not a catalog
/// entity, and the entity path switches exhaustively over the kind on both sides.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PlaylistStats {
    pub id: Uuid,
    pub period: Period,
    pub window_start: EpochMillis,
    pub window_end: EpochMillis,
    pub granularity: BucketGranularity,
    /// Whose plays these figures cover. Echoed back so a panel can label itself honestly rather
    /// than trusting the toggle it sent.
    #[serde(default)]
    pub scope: StatsScope,
    pub total_plays: u32,
    pub total_ms_played: u64,
    /// First/last play attributed to this playlist (epoch millis). `None` if never played.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_played: Option<EpochMillis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played: Option<EpochMillis>,
    /// Distinct listeners in the window. Only meaningful — and only populated — for
    /// `StatsScope::Global`; always `None` for `Me`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_listeners: Option<u32>,
    /// Hub-wide all-time plays from the playlist's global rollup. `None` until the rollup has a row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_plays: Option<u64>,
    /// Local-calendar play trend (chronological), for a sparkline.
    pub trend: Vec<TimeBucket>,
    /// Most-played tracks *from this playlist* in the window.
    pub top_tracks: Vec<TopItem>,
    /// When playlist attribution began (epoch millis). Plays recorded before this have no playlist
    /// context and can never be attributed, so the UI must state the limitation.
    pub tracking_since: EpochMillis,
}

/// Lightweight "recently played" feed item for the home view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RecentPlay {
    /// The play event's idempotency id. Always present, so it is the stable list key.
    pub event_id: Uuid,
    /// Resolved catalog track id, so the row can link to the track. `None` when the scrobble never
    /// matched the catalog — which is why this cannot double as the list key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<Uuid>,
    /// Resolved primary artist id, when the scrobble matched the catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<Uuid>,
    /// Resolved album id, when the scrobble matched the catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_id: Option<Uuid>,
    pub title: String,
    pub artist: String,
    /// Catalog cover resolved like history rows: track cover first, then album cover, through the
    /// Hub image endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub played_at: EpochMillis,
}
