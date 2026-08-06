//! Track identification: an acoustic fingerprint in, a MusicBrainz identity out.
//!
//! # Why this crosses the Hub boundary at all
//!
//! The defining rule of Chordia is that **the Hub never sees audio bytes**. This contract does not
//! break that rule, because [`IdentifyRequest::fingerprint`] is *not* audio. It is a Chromaprint
//! string: a lossy acoustic hash, computed locally by `fpcalc`, that reduces a whole track to a few
//! hundred bytes describing its coarse spectral shape over time. It is one-way — no decoder can
//! reconstruct anything listenable from it — and it is exactly what AcoustID's web service takes as
//! input. So the work splits along the byte boundary:
//!
//! - **Fingerprint computation stays in the library.** Only the library host ever opens the file.
//! - **The lookup moves to the Hub.** Fingerprint + duration → AcoustID → MusicBrainz identity is an
//!   API call carrying a hash and some tag hints, never a sample. Centralizing it is safe.
//!
//! # Why the Hub, and not each library
//!
//! AcoustID requires an API key. When identification lived entirely in the library it was gated
//! behind that library's own `[acoustid] api_key`, which essentially no self-hosted instance sets —
//! so the feature was off by default for everyone, and untagged imports (a FLAC carrying only
//! ARTIST and TITLE) landed with no album and no artwork forever. The Hub already owns third-party
//! provider access: it holds the MusicBrainz and fanart.tv credentials, the shared rate limiter, the
//! response cache, and the required `User-Agent`. Moving the lookup there means **a self-hoster needs
//! no AcoustID key and no config at all** — identification is simply on.
//!
//! # Absent means "not resolved"
//!
//! Every optional field below means *the Hub could not resolve this*, and it is never serialized as
//! an empty value. A producer must emit `None` — never `Some("")`, never `Some(0)` — and a consumer
//! must treat an absent field as "leave whatever I already have alone" (i.e. `COALESCE`), never as
//! an instruction to blank a column. A partial identification is the normal case, not an error:
//! resolving `recording_mbid` and `album` while `track_no` stays unknown is a useful result.
//!
//! # No match versus provider failure
//!
//! These are different answers and the contract keeps them different. A fingerprint AcoustID simply
//! does not know yields **no [`IdentifyResponse`] at all** (`204 No Content`) — the library may retry
//! later or give up cheaply. A provider that is down, rate-limited, or misconfigured yields an
//! **error status**, which the library must log and retry. Collapsing the second into the first is
//! how a dead provider once stayed invisible for four days; the shape here makes that collapse
//! impossible, since [`IdentifyResponse::acoustid`] is non-optional and a response either exists in
//! full or does not exist.
//!
//! # Why there is no batch shape
//!
//! Deliberate. The library's identification pass is paced by `fpcalc`, which is local, serial, and
//! far slower than the network hop — so batching the Hub call wins no throughput. Meanwhile the Hub
//! must rate-limit AcoustID *globally across every library*, which it does with an internal queue
//! whether it is handed one fingerprint or twenty-five; a batch would just hold one HTTP request
//! open for the whole queue wait, inviting timeouts and retries that duplicate upstream quota. A
//! batch also forces a per-item status to keep "no match" distinct from "provider failed", which one
//! fingerprint per request gets for free from the HTTP status. Should batching ever pay for itself,
//! it can be added additively alongside these types without changing them.
//!
//! # Compatibility
//!
//! Additive and serde-defaulted throughout. An older library that sends only a fingerprint and a
//! duration is a valid request against a newer Hub, and unknown fields from a newer peer are ignored
//! rather than rejected, so `contracts → backend → library` deploys in that order without a flag day.

use serde::{Deserialize, Serialize};

/// Body of `POST /v1/catalog/identify`: one track's acoustic fingerprint plus whatever the file already
/// claims about itself.
///
/// The hints are not filters — the Hub always asks AcoustID about the fingerprint. They break ties
/// among the competing recordings a single AcoustID id can map to (one song appears on an album, a
/// deluxe edition, three compilations and a live record, each a different MusicBrainz recording with
/// a different track position). Matching the hinted album is the strongest of those signals: it is
/// what makes a FLAC and an MP3 of the same track converge on one `recording_mbid`. All three are
/// optional and are routinely absent — an untagged import with no ALBUM tag is precisely the case
/// this endpoint exists to rescue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IdentifyRequest {
    /// The Chromaprint fingerprint as emitted by `fpcalc -json` (base64-ish, URL-safe).
    ///
    /// A lossy acoustic hash of the decoded audio, **not audio** — see the module docs. This is the
    /// only reason an identification request may leave the library host at all.
    pub fingerprint: String,
    /// Decoded duration in milliseconds, as reported by `fpcalc` alongside the fingerprint.
    ///
    /// AcoustID matches on duration as well as fingerprint, so this must be the duration `fpcalc`
    /// measured, not one read from a tag. Milliseconds to match every other duration in these
    /// contracts; the Hub converts to the whole seconds AcoustID expects.
    pub duration_ms: u32,
    /// The file's current TITLE tag, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The file's current ARTIST tag, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    /// The file's current ALBUM tag, if any. The strongest disambiguation hint when present, and the
    /// one most often missing on the imports that need identifying.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
}

/// Response of `POST /v1/catalog/identify`: the identity AcoustID and MusicBrainz agree on for a
/// fingerprint, as far as it could be resolved.
///
/// Only [`acoustid`](Self::acoustid) is guaranteed. Everything else is `None` when unresolved and is
/// omitted from the JSON entirely — see the module docs: absent means "not resolved", so a consumer
/// backfills (`COALESCE`) and never overwrites a known value with a blank. If AcoustID knows the
/// fingerprint but has no linked recording, an `IdentifyResponse` with only `acoustid` set is still
/// worth storing: the AcoustID id alone is the strongest own-copy match layer, since two encodings
/// of one recording share it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IdentifyResponse {
    /// The AcoustID id of the best-scoring match. Stable across encodings of the same recording,
    /// which is what makes it the top layer of [`crate::catalog::TrackFingerprint`].
    pub acoustid: String,
    /// MusicBrainz Recording id of the chosen recording. `None` when the AcoustID id has no linked
    /// recording yet (a real and fairly common state for new or obscure releases).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_mbid: Option<String>,
    /// Canonical album title of the matched release, from MusicBrainz rather than from the file.
    /// This is the field that gives an `album_id IS NULL` import an album at all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    /// MusicBrainz Release id of the matched release (the specific edition, not the release group).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_mbid: Option<String>,
    /// Artist credited for the matched *release* — the album artist, which on a compilation or a
    /// split differs from the track's own artist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub album_artist: Option<String>,
    /// Canonical recording title from MusicBrainz. Useful even when the file has a TITLE tag, since
    /// the canonical form is what other libraries will have normalized against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Track position on the matched release's medium (1-based). `None` when the release lists no
    /// position for this recording.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_no: Option<u16>,
    /// Medium (disc) position on the matched release (1-based).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disc_no: Option<u16>,
    /// Release year of the matched release. `None` when MusicBrainz has no date, a partial date
    /// without a year, or a year outside the representable range — the Hub drops such values rather
    /// than truncating them into a plausible-looking wrong answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Absent must stay absent on the wire. If any optional field ever serialized as `null` or `""`,
    /// a consumer doing a naive write would blank a column it should have left alone.
    #[test]
    fn unresolved_fields_are_omitted_not_blanked() {
        let partial = IdentifyResponse {
            acoustid: "acid-1".into(),
            ..Default::default()
        };
        let json = serde_json::to_value(&partial).unwrap();
        assert_eq!(json, serde_json::json!({ "acoustid": "acid-1" }));
    }

    /// A fully untagged file — the case this endpoint exists for — must be a valid request with no
    /// hints at all, and must not acquire empty-string hints on the way through.
    #[test]
    fn untagged_file_needs_only_fingerprint_and_duration() {
        let req: IdentifyRequest =
            serde_json::from_str(r#"{"fingerprint":"AQABz0","duration_ms":215000}"#).unwrap();
        assert_eq!(req.title, None);
        assert_eq!(req.artist, None);
        assert_eq!(req.album, None);
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            serde_json::json!({ "fingerprint": "AQABz0", "duration_ms": 215000 })
        );
    }

    /// Forward compatibility: an older Hub must accept a request from a newer library that carries a
    /// field it has never heard of, rather than 400-ing the whole identification pass.
    #[test]
    fn unknown_fields_are_ignored() {
        let req: IdentifyRequest = serde_json::from_str(
            r#"{"fingerprint":"AQABz0","duration_ms":215000,"future_hint":"x"}"#,
        )
        .unwrap();
        assert_eq!(req.fingerprint, "AQABz0");
    }
}
