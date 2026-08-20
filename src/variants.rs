//! Markers lifted out of a track title.
//!
//! Tag titles arrive carrying a parenthetical tail that says something about the recording or the
//! release rather than naming the song: `Song (Album Version)`, `Song (Album Version (Explicit))`,
//! `Song - Remastered 2011`. It is noise in a list of a thousand tracks, it is repeated on every
//! row of an album, and one of the most common suffixes — `(Explicit)` — duplicates a field the
//! track already has.
//!
//! So the tail is parsed once, on the way into the catalog: the title keeps the song's name and
//! what was stripped becomes a typed list the client renders as small badges, the same way the
//! explicit badge already works. Nothing is guessed. A parenthetical this does not recognise is
//! left in the title untouched, because an unknown suffix is far more likely to be part of the name
//! than a marker worth inventing a meaning for.

use serde::{Deserialize, Serialize};

/// One marker stripped from a title.
///
/// Ordered by how much it changes the recording: the first few are genuinely different performances
/// and the last few describe the release a track was on. That ordering is what the client renders
/// in, so the badge that matters most is the one nearest the title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum TrackVariant {
    Live,
    Acoustic,
    Instrumental,
    Remix,
    Demo,
    Cover,
    Karaoke,
    Extended,
    RadioEdit,
    SingleVersion,
    Remaster,
    Bonus,
    Deluxe,
}

impl TrackVariant {
    /// The wire spelling, and the i18n key suffix the client renders from.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Acoustic => "acoustic",
            Self::Instrumental => "instrumental",
            Self::Remix => "remix",
            Self::Demo => "demo",
            Self::Cover => "cover",
            Self::Karaoke => "karaoke",
            Self::Extended => "extended",
            Self::RadioEdit => "radio_edit",
            Self::SingleVersion => "single_version",
            Self::Remaster => "remaster",
            Self::Bonus => "bonus",
            Self::Deluxe => "deluxe",
        }
    }

    /// Parse a wire spelling back. Used by the smart-playlist resolver, which stores a rule's value
    /// as text.
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "live" => Self::Live,
            "acoustic" => Self::Acoustic,
            "instrumental" => Self::Instrumental,
            "remix" => Self::Remix,
            "demo" => Self::Demo,
            "cover" => Self::Cover,
            "karaoke" => Self::Karaoke,
            "extended" => Self::Extended,
            "radio_edit" => Self::RadioEdit,
            "single_version" => Self::SingleVersion,
            "remaster" => Self::Remaster,
            "bonus" => Self::Bonus,
            "deluxe" => Self::Deluxe,
            _ => return None,
        })
    }
}

/// What one parenthetical turned out to be.
enum Parsed {
    /// A marker worth keeping as a badge.
    Variant(TrackVariant),
    /// Recognised, but says nothing a reader wants: `(Album Version)`.
    Noise,
    /// A content rating that belongs in `advisory`, not in the name.
    Advisory(&'static str),
    /// Not recognised. Stays in the title.
    Keep,
}

/// What a title looked like once its markers were taken out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrippedTitle {
    pub title: String,
    pub variants: Vec<TrackVariant>,
    /// `"explicit"` or `"clean"`, when the TITLE said so. The caller decides whether to use it —
    /// a rating tag on the file is better evidence and should win.
    pub advisory: Option<&'static str>,
}

/// Classify the inside of one parenthetical (or one trailing dash segment).
///
/// Matching is on normalised words, so `Remastered 2011`, `2011 Remaster` and `Digital Remaster`
/// all land on the same marker without needing an entry each. Deliberately substring-based on a
/// small vocabulary rather than a general parser: the input is human-typed tag text with no grammar
/// to rely on, and the cost of a wrong guess is a mangled song title.
fn classify(segment: &str) -> Parsed {
    let s = segment.trim().to_ascii_lowercase();
    if s.is_empty() {
        return Parsed::Keep;
    }

    // A featured-artist credit is NEVER touched. The credited-artist line is rebuilt from
    // `track_artists`, and a guest who is only named in the title would vanish entirely.

    // A featured-artist credit is NEVER touched. The credited-artist line is rebuilt from
    // `track_artists`, and a guest who is only named in the title would vanish entirely.
    // A featured-artist credit is NEVER touched. The credited-artist line is rebuilt from
    // `track_artists`, and a guest who is only named in the title would vanish entirely.
    if s.starts_with("feat") || s.starts_with("ft.") || s.starts_with("with ") {
        return Parsed::Keep;
    }

    // Ratings first: `(Album Version (Explicit))` arrives here as `album version (explicit)` after
    // the outer bracket is peeled, so the rating has to be found before the noise check consumes it.
    if s.contains("explicit") {
        return Parsed::Advisory("explicit");
    }
    if s.contains("clean") {
        return Parsed::Advisory("clean");
    }

    // Noise: true of almost every track on almost every album, and therefore information-free.
    if s.contains("album version") || s.contains("album edit") || s == "original" {
        return Parsed::Noise;
    }

    // Order matters where words overlap. `live` is checked as a whole word because "olive" and
    // "delivery" are real title words; the rest are distinctive enough to match anywhere.
    if s.contains("remaster") {
        return Parsed::Variant(TrackVariant::Remaster);
    }
    if s.contains("radio edit") || s.contains("radio version") {
        return Parsed::Variant(TrackVariant::RadioEdit);
    }
    if s.contains("single version") || s.contains("single edit") {
        return Parsed::Variant(TrackVariant::SingleVersion);
    }
    if s.contains("instrumental") {
        return Parsed::Variant(TrackVariant::Instrumental);
    }
    if s.contains("acoustic") || s.contains("unplugged") {
        return Parsed::Variant(TrackVariant::Acoustic);
    }
    if s.contains("karaoke") {
        return Parsed::Variant(TrackVariant::Karaoke);
    }
    if s.contains("remix") || s.contains(" mix") || s.ends_with("mix") {
        return Parsed::Variant(TrackVariant::Remix);
    }
    if s.contains("demo") {
        return Parsed::Variant(TrackVariant::Demo);
    }
    if s.contains("extended") {
        return Parsed::Variant(TrackVariant::Extended);
    }
    if s.contains("bonus") {
        return Parsed::Variant(TrackVariant::Bonus);
    }
    if s.contains("deluxe") {
        return Parsed::Variant(TrackVariant::Deluxe);
    }
    if s.contains("cover version") || s.starts_with("cover") {
        return Parsed::Variant(TrackVariant::Cover);
    }
    if s.split(|c: char| !c.is_alphanumeric()).any(|w| w == "live") {
        return Parsed::Variant(TrackVariant::Live);
    }

    Parsed::Keep
}

/// Take the markers out of a title.
///
/// Only TRAILING segments are considered. A parenthetical in the middle of a name is part of the
/// name — `(Don't Fear) The Reaper` is the song, not a marker on it — and walking backwards from
/// the end is what keeps that true without needing to understand the rest of the string.
pub fn strip_title(raw: &str) -> StrippedTitle {
    let mut title = raw.trim().to_string();
    let mut variants: Vec<TrackVariant> = Vec::new();
    let mut advisory: Option<&'static str> = None;
    let mut changed_any = false;

    loop {
        let trimmed = title.trim_end();
        // A bracketed tail: `... (Live)` or `... [Live]`.
        let bracket = trimmed.chars().last().and_then(|c| match c {
            ')' => Some(('(', ')')),
            ']' => Some(('[', ']')),
            _ => None,
        });
        let (segment, head) = if let Some((open, close)) = bracket {
            // Match the bracket that opened this tail, counting depth, so the nested
            // `(Album Version (Explicit))` is taken as ONE segment rather than split at the inner
            // bracket and left with a stray `(Album Version`.
            let bytes: Vec<char> = trimmed.chars().collect();
            let mut depth = 0i32;
            let mut start = None;
            for (i, c) in bytes.iter().enumerate().rev() {
                if *c == close {
                    depth += 1;
                } else if *c == open {
                    depth -= 1;
                    if depth == 0 {
                        start = Some(i);
                        break;
                    }
                }
            }
            let Some(start) = start else { break };
            let inner: String = bytes[start + 1..bytes.len() - 1].iter().collect();
            let head: String = bytes[..start].iter().collect();
            (inner, head)
        } else if let Some((head, tail)) = trimmed.rsplit_once(" - ") {
            // The other common shape, and the reason it is restricted to ` - ` with spaces: a
            // hyphenated word ("Rock-A-Bye") must not be read as a marker boundary.
            (tail.to_string(), head.to_string())
        } else {
            break;
        };

        // A nested rating rides along with its outer segment: classify the whole thing, then look
        // inside for a rating the outer classification did not already report.
        match classify(&segment) {
            Parsed::Keep => break,
            Parsed::Noise => {}
            Parsed::Advisory(a) => advisory = advisory.or(Some(a)),
            Parsed::Variant(v) => {
                if !variants.contains(&v) {
                    variants.push(v);
                }
            }
        }
        // `(Album Version (Explicit))` classifies as Advisory on the inner word; the outer
        // `album version` is noise either way, so both are consumed together and nothing is lost.
        if advisory.is_none() {
            if segment.to_ascii_lowercase().contains("explicit") {
                advisory = Some("explicit");
            } else if segment.to_ascii_lowercase().contains("clean") {
                advisory = Some("clean");
            }
        }
        title = head.trim_end().to_string();
        changed_any = true;
        if title.is_empty() {
            break;
        }
    }

    // A title that was ONLY a marker is not an improvement over the original. Rare, but the failure
    // mode is a row with a blank name and no way to tell what it was.
    if title.trim().is_empty() {
        return StrippedTitle {
            title: raw.trim().to_string(),
            variants: Vec::new(),
            advisory: None,
        };
    }

    if changed_any {
        variants.sort();
    }
    StrippedTitle {
        title,
        variants,
        advisory,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(raw: &str) -> (String, Vec<&'static str>, Option<&'static str>) {
        let r = strip_title(raw);
        (
            r.title,
            r.variants.into_iter().map(TrackVariant::as_str).collect(),
            r.advisory,
        )
    }

    #[test]
    fn strips_the_suffix_that_says_nothing() {
        assert_eq!(
            s("Bad Blood (Album Version)"),
            ("Bad Blood".into(), vec![], None)
        );
    }

    #[test]
    fn reads_a_nested_rating_and_keeps_it() {
        // The shape that prompted all of this. Both brackets go, and the rating is not lost with
        // them — it moves to the field that already exists for it.
        assert_eq!(
            s("Bad Blood (Album Version (Explicit))"),
            ("Bad Blood".into(), vec![], Some("explicit"))
        );
    }

    #[test]
    fn keeps_a_parenthetical_that_is_part_of_the_name() {
        // The whole reason classification is an allowlist. Guessing here renames the song.
        assert_eq!(
            s("(Don't Fear) The Reaper"),
            ("(Don't Fear) The Reaper".into(), vec![], None)
        );
        assert_eq!(
            s("Everything In Its Right Place"),
            ("Everything In Its Right Place".into(), vec![], None)
        );
    }

    #[test]
    fn never_touches_a_featured_credit() {
        // The credit line is rebuilt from `track_artists`, but a guest named only in the title
        // would disappear completely, and there is no way to get them back.
        //
        // Both fixtures are bands whose NAME contains a marker word, which is the only case where
        // the guard does any work. "feat. Nicki Minaj" would survive without it too, and a test
        // built on that would pass whether the guard existed or not.
        assert_eq!(
            s("Song (with Acoustic Alchemy)"),
            ("Song (with Acoustic Alchemy)".into(), vec![], None)
        );
        assert_eq!(
            s("Song (feat. Cover Drive)"),
            ("Song (feat. Cover Drive)".into(), vec![], None)
        );
    }

    #[test]
    fn recognises_a_remaster_however_it_is_written() {
        for raw in [
            "Come Together - Remastered 2009",
            "Come Together (2009 Remaster)",
            "Come Together (Digital Remaster)",
        ] {
            assert_eq!(
                s(raw),
                ("Come Together".into(), vec!["remaster"], None),
                "{raw}"
            );
        }
    }

    #[test]
    fn collects_several_markers_from_one_title() {
        let (title, variants, _) = s("Song (Live) (Remastered)");
        assert_eq!(title, "Song");
        // Sorted by how much the marker changes the recording, so the client renders the one that
        // matters first rather than in tag order.
        assert_eq!(variants, vec!["live", "remaster"]);
    }

    #[test]
    fn does_not_read_live_out_of_an_ordinary_word() {
        // Whole-word matching, because "delivery" and "olive" contain "live" and a substring test
        // marks both as live recordings.
        //
        // The fixtures must reach `classify`, which only sees a BRACKETED or dashed tail. Plain
        // titles like "Special Delivery" leave the loop on the first pass and would pass this test
        // no matter how `live` is matched — which is what the first version of it did.
        assert_eq!(
            s("Song (Special Delivery)"),
            ("Song (Special Delivery)".into(), vec![], None)
        );
        assert_eq!(
            s("Song - Olive Branch"),
            ("Song - Olive Branch".into(), vec![], None)
        );
        // And the marker itself still reads, so this is not passing by refusing to match anything.
        assert_eq!(s("Song (Live)"), ("Song".into(), vec!["live"], None));
        assert_eq!(
            s("Song - Live at Wembley"),
            ("Song".into(), vec!["live"], None)
        );
    }

    #[test]
    fn a_title_that_is_only_a_marker_is_left_alone() {
        // Stripping everything leaves a nameless row, which is worse than the suffix.
        assert_eq!(s("(Live)"), ("(Live)".into(), vec![], None));
    }

    #[test]
    fn does_not_split_a_hyphenated_word() {
        assert_eq!(
            s("Rock-A-Bye Baby"),
            ("Rock-A-Bye Baby".into(), vec![], None)
        );
    }
}
