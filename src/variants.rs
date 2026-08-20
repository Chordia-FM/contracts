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
//!
//! # A badge only replaces a segment when it says everything the segment said
//!
//! `(Album Version)` is noise: every track on the album is the album version, and a reader learns
//! nothing by being told. `(Gianni Marino Remix)` is not noise — it names the person who made that
//! recording, and for most remixes the title is the ONLY place they are named, because the file's
//! artist tag says "Mike Posner" and stops there. Collapsing that to a remix badge deletes the
//! credit outright, which is the same failure the featured-artist guard below exists to prevent.
//! `(Live at Wembley Arena)` fails the same way: strip two of those and an artist has two rows both
//! called "Song" with a live badge and nothing to tell the takes apart.
//!
//! So a segment is stripped only when what remains of it, once every word the badge already
//! expresses is taken out, is nothing. A leftover is information no badge carries, and the segment
//! stays in the title. The marker is reported either way — recognising a marker and deleting a
//! segment are separate decisions, so a title that keeps `(Gianni Marino Remix)` still filters out
//! of a "no remixes" smart playlist.

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

    /// Parse a wire spelling back. Used by the smart-playlist resolver, which stores a rule's
    /// value as text, and by the row mapper, which reads a `text[]` column.
    ///
    /// Named `from_wire` rather than `from_str`: the latter shadows `std::str::FromStr::from_str`,
    /// which clippy rejects because a reader cannot tell which one a call resolves to. Implementing
    /// the trait proper would buy `"live".parse()` and an error type nobody here wants — every
    /// caller treats an unknown spelling as "no marker", not as a failure.
    pub fn from_wire(s: &str) -> Option<Self> {
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
#[derive(Default)]
struct Parsed {
    /// EVERY marker the segment carries. `(Live Acoustic)` is both, and a track that is only
    /// tagged as one of them is missing from a filter for the other.
    variants: Vec<TrackVariant>,
    /// A content rating that belongs in `advisory`, not in the name.
    advisory: Option<&'static str>,
    /// Recognised at all. `false` means the segment is part of the song's name and stays put —
    /// which is different from a segment that IS recognised and contributes no badge, like
    /// `(Album Version)`.
    known: bool,
}

/// What a title looked like once its markers were taken out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrippedTitle {
    pub title: String,
    pub variants: Vec<TrackVariant>,
    /// `"explicit"` or `"clean"`, when the TITLE said so. The caller decides whether to use it —
    /// a rating tag on the file is better evidence and should win.
    pub advisory: Option<&'static str>,
    /// Who the title says remixed this, when it says so: `(Gianni Marino Remix)` → `Gianni Marino`.
    ///
    /// A remixer is a credited artist — streaming services list them on the artist line, and the
    /// recording is their work as much as the writer's — but the file's artist tag almost never
    /// carries them. The caller credits this name so the DJ is findable, instead of leaving them as
    /// text inside one row's title.
    pub remixer: Option<String>,
    /// Where the title says it was recorded: `(Live at Wembley Arena)` → `Wembley Arena`.
    ///
    /// A hint and named as one. The preposition is what makes it a guess worth making — "Live at X"
    /// is a place, "Live Session" is not — but the parser cannot tell a venue from a city from a
    /// radio studio, and MusicBrainz is the one that can, with coordinates. This is what the catalog
    /// has when MusicBrainz has nothing, which for live bootlegs is most of the time.
    pub venue: Option<String>,
    /// The year a live segment carries: `(Live at Wembley Arena, 1985)` → `1985`.
    ///
    /// Read even when the segment is stripped, because `(Live 1978)` subtracts to nothing and would
    /// otherwise take the only date with it.
    pub recorded_year: Option<u16>,
}

/// Every word a badge already expresses, plus the connective tissue around it.
///
/// The point of the list is subtraction: a segment is safe to delete only when it contains nothing
/// but these. Bare numbers count too — `Remastered 2009` and `2009 Remaster` are one marker written
/// two ways, and the year belongs to the marker rather than being a fact the badge is dropping.
const VOCAB: &[&str] = &[
    "a",
    "acoustic",
    "album",
    "an",
    "bonus",
    "clean",
    "cover",
    "cut",
    "deluxe",
    "demo",
    "digital",
    "edit",
    "edited",
    "edition",
    "explicit",
    "extended",
    "instrumental",
    "karaoke",
    "live",
    "master",
    "mastered",
    "mix",
    "mixed",
    "original",
    "radio",
    "re",
    "recording",
    "remaster",
    "remastered",
    "remix",
    "remixed",
    "rerecorded",
    "single",
    "take",
    "the",
    "track",
    "unplugged",
    "version",
];

/// Words that name a KIND of mix rather than a person.
///
/// `(Club Mix)`, `(Nightcore Mix)` and `(Slowed + Reverb Remix)` each leave a residue that looks
/// exactly like a name to a parser, and crediting them mints artists called "Club" and "Nightcore"
/// which the enrichment worker then chases through MusicBrainz for a photo and a biography. Junk
/// rows in `artists` are far more work to undo than a missing credit is to add, so a residue made
/// entirely of these words is not promoted. One word from outside the list is enough to qualify,
/// which is what keeps `DJ Snake` and `Dub Pistols` intact.
const STYLE_QUALIFIERS: &[&str] = &[
    "bootleg",
    "chopped",
    "club",
    "dance",
    "dirty",
    "dj",
    "dub",
    "festival",
    "flip",
    "full",
    "hard",
    "house",
    "intro",
    "long",
    "main",
    "mashup",
    "medley",
    "night",
    "nightcore",
    "outro",
    "party",
    "refix",
    "remake",
    "reverb",
    "rework",
    "screwed",
    "short",
    "slowed",
    "sped",
    "speed",
    "summer",
    "techno",
    "trance",
    "tribute",
    "vip",
    "workout",
];

/// A word reduced to the letters and digits in it, so `(Explicit)` and `Explicit` compare equal.
fn norm(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Whether a word is one the badge already says, or a year riding along with one.
fn is_vocab(word: &str) -> bool {
    let w = norm(word);
    w.is_empty() || w.chars().all(|c| c.is_ascii_digit()) || VOCAB.contains(&w.as_str())
}

/// The part of a segment the badge does not account for, verbatim.
///
/// Trimmed from both ends rather than filtered word by word, so the middle keeps its own
/// punctuation: `A-Trak` comes back as `A-Trak` instead of being rebuilt into `A Trak`.
fn residue(segment: &str) -> &str {
    let mut words: Vec<(usize, usize)> = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in segment.char_indices() {
        if c.is_whitespace() {
            if let Some(s) = start.take() {
                words.push((s, i));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        words.push((s, segment.len()));
    }

    let mut lo = 0;
    let mut hi = words.len();
    while lo < hi && is_vocab(&segment[words[lo].0..words[lo].1]) {
        lo += 1;
    }
    while hi > lo && is_vocab(&segment[words[hi - 1].0..words[hi - 1].1]) {
        hi -= 1;
    }
    if lo >= hi {
        return "";
    }
    &segment[words[lo].0..words[hi - 1].1]
}

/// The remixer a segment names, if it names one at all.
///
/// Trimmed from the END only, unlike [`residue`]. A name can BEGIN with a word this file also
/// treats as vocabulary — Clean Bandit, The Killers, Cover Drive, Single Mothers — and trimming
/// both ends credits "Bandit" and "Killers". Nothing is called "X Remix" unless that last word is
/// the marker, so the trailing end is safe to eat and the leading end is not.
fn remixer_from(segment: &str) -> Option<String> {
    let mut words: Vec<&str> = segment.split_whitespace().collect();
    while words.last().is_some_and(|w| is_vocab(w)) {
        words.pop();
    }
    // `Remix by Skrillex` and `Remix - Skrillex` put the marker first instead, which is the one
    // shape where a leading word has to go. Only these, and only from the front.
    while words
        .first()
        .is_some_and(|w| matches!(norm(w).as_str(), "remix" | "mix" | "by" | ""))
    {
        words.remove(0);
    }
    let name = words
        .join(" ")
        .trim_matches(|c: char| c == ',' || c == '-' || c == '+' || c == '&' || c.is_whitespace())
        .to_string();
    if name.is_empty() {
        return None;
    }
    // Longer than this is a sentence, a second title, or a tracklist glued on by a careless rip.
    // It is not somebody's name, and the cost of being wrong is a permanent junk artist.
    if name.split_whitespace().count() > 5 || name.chars().count() > 60 {
        return None;
    }
    if name
        .split_whitespace()
        .all(|w| STYLE_QUALIFIERS.contains(&norm(w).as_str()))
    {
        return None;
    }
    Some(name)
}

/// The place a live segment names, if it names one at all.
///
/// A preposition is required, and it is the whole guard: "Live at Wembley Arena" is somewhere,
/// "Live Session" and "Live Acoustic" are not. Without it the residue of any live segment would be
/// read as a location, and the map this feeds would fill up with places called "Session".
fn venue_from(left: &str) -> Option<String> {
    let lower = left.to_ascii_lowercase();
    let rest = ["at ", "in ", "from ", "@ "]
        .iter()
        .find_map(|p| lower.starts_with(p).then(|| &left[p.len()..]))?;

    // A trailing year belongs to `recorded_year`, which reads it off the whole segment; leaving it
    // on the venue would make "Wembley Arena 1985" and "Wembley Arena 1986" two different places.
    let mut end = rest.len();
    loop {
        let head =
            rest[..end].trim_end_matches(|c: char| c == ',' || c == '-' || c.is_whitespace());
        let last = head.split_whitespace().next_back()?;
        if last.chars().all(|c| c.is_ascii_digit()) {
            end = head.len() - last.len();
        } else {
            end = head.len();
            break;
        }
    }
    let name = rest[..end].trim();
    if name.is_empty() {
        return None;
    }
    // Beyond this it is a sentence or a tracklist, not a place.
    if name.split_whitespace().count() > 8 || name.chars().count() > 80 {
        return None;
    }
    Some(name.to_string())
}

/// A four-digit year standing alone anywhere in a segment.
fn year_in(segment: &str) -> Option<u16> {
    segment
        .split(|c: char| !c.is_ascii_digit())
        .filter(|w| w.len() == 4)
        .find_map(|w| w.parse::<u16>().ok())
        .filter(|y| (1900..=2099).contains(y))
}

/// Classify the inside of one parenthetical (or one trailing dash segment).
///
/// Every marker that applies, not the first one found. A segment saying "Live Acoustic" describes a
/// recording that is both, and reporting only one of them hides the track from a filter for the
/// other — which is the same information loss as deleting the words would be.
///
/// The exception is the family answering "which edit is this": radio edit, single version, remix,
/// extended. Those are alternatives and they overlap on the same words — "Radio Mix" is a radio
/// edit and NOT also a remix, "Extended Mix" is one badge rather than two. At most one is taken,
/// most specific first. Everything else is asked independently.
///
/// Matching is on normalised words, so `Remastered 2011`, `2011 Remaster` and `Digital Remaster`
/// all land on the same marker without needing an entry each. Deliberately substring-based on a
/// small vocabulary rather than a general parser: the input is human-typed tag text with no grammar
/// to rely on, and the cost of a wrong guess is a mangled song title.
fn classify(segment: &str) -> Parsed {
    let mut out = Parsed::default();
    let s = segment.trim().to_ascii_lowercase();
    if s.is_empty() {
        return out;
    }

    // A featured-artist credit is NEVER touched. The credited-artist line is rebuilt from
    // `track_artists`, and a guest who is only named in the title would vanish entirely.
    if s.starts_with("feat") || s.starts_with("ft.") || s.starts_with("with ") {
        return out;
    }

    // A rating, but only when the segment says nothing else. `explicit` and `clean` are words that
    // turn up inside names — "Clean Bandit Remix" is a remix by a band, not a clean edit — and an
    // empty residue is exactly the test for "this segment is the marker and no more".
    if residue(segment).is_empty() {
        if s.contains("explicit") {
            out.advisory = Some("explicit");
            out.known = true;
        } else if s.contains("clean") {
            out.advisory = Some("clean");
            out.known = true;
        }
    }

    // Which edit is this: at most one, most specific first.
    //
    // `Original Mix` leads the list and contributes nothing, because in dance music it is what a
    // track is called when it has NOT been remixed — the thing every remix is a remix of. Reading
    // it as a remix inverts the meaning and makes "no remixes" throw away precisely the originals.
    // `Album Version` is the same shape: true of nearly every track on nearly every album, and so
    // information-free.
    if s.contains("album version")
        || s.contains("album edit")
        || s.contains("original mix")
        || s.contains("original version")
        || s == "original"
    {
        out.known = true;
    } else if s.contains("radio edit") || s.contains("radio version") || s.contains("radio mix") {
        out.variants.push(TrackVariant::RadioEdit);
    } else if s.contains("single version") || s.contains("single edit") {
        out.variants.push(TrackVariant::SingleVersion);
    } else if s.contains("remix") || s.contains(" mix") || s.ends_with("mix") {
        out.variants.push(TrackVariant::Remix);
    } else if s.contains("extended") {
        out.variants.push(TrackVariant::Extended);
    }

    // And every marker that stands on its own. `live` is checked as a whole word because "olive"
    // and "delivery" are real title words; the rest are distinctive enough to match anywhere.
    if s.contains("remaster") {
        out.variants.push(TrackVariant::Remaster);
    }
    if s.contains("instrumental") {
        out.variants.push(TrackVariant::Instrumental);
    }
    if s.contains("acoustic") || s.contains("unplugged") {
        out.variants.push(TrackVariant::Acoustic);
    }
    if s.contains("karaoke") {
        out.variants.push(TrackVariant::Karaoke);
    }
    if s.contains("demo") {
        out.variants.push(TrackVariant::Demo);
    }
    if s.contains("bonus") {
        out.variants.push(TrackVariant::Bonus);
    }
    if s.contains("deluxe") {
        out.variants.push(TrackVariant::Deluxe);
    }
    if s.contains("cover version") || s.starts_with("cover") {
        out.variants.push(TrackVariant::Cover);
    }
    if s.split(|c: char| !c.is_alphanumeric()).any(|w| w == "live") {
        out.variants.push(TrackVariant::Live);
    }

    out.known |= !out.variants.is_empty();
    out
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
    let mut remixer: Option<String> = None;
    let mut venue: Option<String> = None;
    let mut recorded_year: Option<u16> = None;

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

        let parsed = classify(&segment);
        if !parsed.known {
            break;
        }
        for v in &parsed.variants {
            if !variants.contains(v) {
                variants.push(*v);
            }
        }
        advisory = advisory.or(parsed.advisory);
        // Read whether or not the segment survives: `(Live 1978)` subtracts to nothing and would
        // take the only date on the recording with it.
        if parsed.variants.contains(&TrackVariant::Live) && recorded_year.is_none() {
            recorded_year = year_in(&segment);
        }

        // The marker is recorded either way; whether the SEGMENT goes depends on whether the badge
        // covered all of it. A leftover is a venue, a date, a mix name or a person, and none of
        // those survive as a badge. Stop rather than continue, because anything further left now
        // sits behind text that is staying.
        let left = residue(&segment);
        if !left.is_empty() {
            // Both, because one segment can carry both: "(Live at Wembley, Kaskade Remix)".
            if parsed.variants.contains(&TrackVariant::Remix) && remixer.is_none() {
                remixer = remixer_from(&segment);
            }
            if parsed.variants.contains(&TrackVariant::Live) && venue.is_none() {
                venue = venue_from(left);
            }
            break;
        }

        title = head.trim_end().to_string();
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
            remixer: None,
            venue: None,
            recorded_year: None,
        };
    }

    variants.sort();
    StrippedTitle {
        title,
        variants,
        advisory,
        remixer,
        venue,
        recorded_year,
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

    fn remixer(raw: &str) -> Option<String> {
        strip_title(raw).remixer
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
        assert_eq!(s("Song - Live"), ("Song".into(), vec!["live"], None));
    }

    #[test]
    fn every_marker_the_parser_knows_can_actually_be_stripped() {
        // `classify` recognises a marker by word; `residue` decides whether the segment goes. A word
        // one knows and the other does not leaves a residue equal to the marker itself, so the badge
        // appears and the suffix stays forever — visible only as a title nobody notices is still
        // dirty. That is exactly how "(Bonus)" survived a rewrite of this file with every other test
        // in it still green, and it was a real library that caught it rather than this suite.
        //
        // One row per branch in `classify`, including the alternate spellings, because the failure
        // is per-word and not per-variant.
        for (segment, want) in [
            ("Live", TrackVariant::Live),
            ("Acoustic", TrackVariant::Acoustic),
            ("Unplugged", TrackVariant::Acoustic),
            ("Instrumental", TrackVariant::Instrumental),
            ("Remix", TrackVariant::Remix),
            ("Extended Mix", TrackVariant::Remix),
            ("Demo", TrackVariant::Demo),
            ("Cover", TrackVariant::Cover),
            ("Cover Version", TrackVariant::Cover),
            ("Karaoke", TrackVariant::Karaoke),
            ("Extended", TrackVariant::Extended),
            ("Radio Edit", TrackVariant::RadioEdit),
            ("Radio Version", TrackVariant::RadioEdit),
            ("Single Version", TrackVariant::SingleVersion),
            ("Single Edit", TrackVariant::SingleVersion),
            ("Remastered", TrackVariant::Remaster),
            ("Digital Remaster", TrackVariant::Remaster),
            ("Bonus", TrackVariant::Bonus),
            ("Bonus Track", TrackVariant::Bonus),
            ("Deluxe", TrackVariant::Deluxe),
            ("Deluxe Edition", TrackVariant::Deluxe),
        ] {
            let r = strip_title(&format!("Song ({segment})"));
            assert_eq!(r.title, "Song", "({segment}) recognised but not stripped");
            assert_eq!(r.variants, vec![want], "({segment})");
        }
        // And the noise branch, which strips to no badge at all.
        for segment in [
            "Album Version",
            "Album Edit",
            "Original Mix",
            "Original Version",
        ] {
            let r = strip_title(&format!("Song ({segment})"));
            assert_eq!(r.title, "Song", "({segment})");
            assert!(r.variants.is_empty(), "({segment}) {:?}", r.variants);
        }
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

    // ── What the badge is allowed to swallow ────────────────────────────────────────────────────

    #[test]
    fn a_named_remixer_stays_in_the_title() {
        // The reported case. "Mike Posner" is the whole artist tag, so the title is the only place
        // Gianni Marino appears anywhere in the file; a bare "remix" badge deletes him.
        assert_eq!(
            s("Bow Chicka Wow Wow (Gianni Marino Remix)"),
            (
                "Bow Chicka Wow Wow (Gianni Marino Remix)".into(),
                vec!["remix"],
                None
            )
        );
        // Same for the dashed spelling, which is how streaming services write it.
        assert_eq!(
            s("Bow Chicka Wow Wow - Gianni Marino Remix"),
            (
                "Bow Chicka Wow Wow - Gianni Marino Remix".into(),
                vec!["remix"],
                None
            )
        );
    }

    #[test]
    fn a_named_remixer_is_reported_as_a_credit() {
        assert_eq!(
            remixer("Bow Chicka Wow Wow (Gianni Marino Remix)").as_deref(),
            Some("Gianni Marino")
        );
        assert_eq!(
            remixer("Levels (Skrillex Remix)").as_deref(),
            Some("Skrillex")
        );
        // The marker word can sit anywhere in the segment, and the rest of it is still the name.
        assert_eq!(
            remixer("Levels (Remix by Skrillex)").as_deref(),
            Some("Skrillex")
        );
        // Two marker words, one name.
        assert_eq!(
            remixer("Song (Kaskade Extended Mix)").as_deref(),
            Some("Kaskade")
        );
        // Punctuation inside the name survives, because the residue is taken verbatim rather than
        // rebuilt out of split words.
        assert_eq!(remixer("Song (A-Trak Remix)").as_deref(), Some("A-Trak"));
    }

    #[test]
    fn a_kind_of_mix_is_not_a_person() {
        // These leave a residue that looks exactly like a name. Crediting them mints an artist
        // called "Club" that enrichment then hunts for a photograph.
        for raw in [
            "Song (Club Mix)",
            "Song (Nightcore Mix)",
            "Song (Slowed Reverb Remix)",
            "Song (Bootleg Mix)",
        ] {
            assert_eq!(remixer(raw), None, "{raw}");
            // Still recognised as a remix, and still kept in the title — the residue says something
            // the badge cannot, even though it is not a credit.
            assert_eq!(
                strip_title(raw).variants,
                vec![TrackVariant::Remix],
                "{raw}"
            );
            assert_eq!(strip_title(raw).title, raw, "{raw}");
        }
        // One word from outside the qualifier list is enough: "DJ" alone is not a name, "DJ Snake"
        // is. A rule that rejected any residue containing a qualifier would lose him.
        assert_eq!(
            remixer("Song (DJ Snake Remix)").as_deref(),
            Some("DJ Snake")
        );
    }

    #[test]
    fn an_unnamed_remix_is_still_stripped() {
        // Nothing is lost by removing this one, which is the whole test: the conservative rule must
        // not turn into "never strip anything".
        assert_eq!(s("Song (Remix)"), ("Song".into(), vec!["remix"], None));
        assert_eq!(
            s("Song (Extended Mix)"),
            ("Song".into(), vec!["remix"], None)
        );
    }

    #[test]
    fn an_original_mix_is_not_a_remix() {
        // In dance music this is what a track is called when it has NOT been remixed. Reading it as
        // a remix inverts the meaning, and a "no remixes" smart playlist would drop the originals
        // and keep everything else.
        assert_eq!(s("Song (Original Mix)"), ("Song".into(), vec![], None));
        assert_eq!(remixer("Song (Original Mix)"), None);
    }

    #[test]
    fn a_live_recording_keeps_where_it_was_recorded() {
        // Two of these strip down to the same row: "Song" with a live badge, twice, and no way to
        // tell the takes apart. The venue and the date are the identity of a live recording.
        assert_eq!(
            s("Song (Live at Wembley Arena)"),
            ("Song (Live at Wembley Arena)".into(), vec!["live"], None)
        );
        assert_eq!(
            s("Song - Live at Budokan"),
            ("Song - Live at Budokan".into(), vec!["live"], None)
        );
    }

    #[test]
    fn a_live_recording_reports_where_and_when() {
        // The point of reading these out rather than leaving them in the title: a venue with
        // coordinates can be counted, grouped and put on a map, and a substring of a title cannot.
        let r = strip_title("Song (Live at Wembley Arena)");
        assert_eq!(r.venue.as_deref(), Some("Wembley Arena"));
        assert_eq!(r.recorded_year, None);

        let r = strip_title("Song - Live at Budokan, 1978");
        assert_eq!(r.venue.as_deref(), Some("Budokan"));
        assert_eq!(r.recorded_year, Some(1978));

        // The year is part of the marker for stripping purposes, so this segment subtracts to
        // nothing and goes — the date has to be read before that happens or it leaves with it.
        let r = strip_title("Song (Live 1978)");
        assert_eq!(r.title, "Song");
        assert_eq!(r.recorded_year, Some(1978));
        assert_eq!(r.venue, None);

        for (raw, want) in [
            ("Song (Live in Paris)", "Paris"),
            ("Song (Live from Abbey Road)", "Abbey Road"),
            (
                "Song (Live at the O2 Arena, London 2019)",
                "the O2 Arena, London",
            ),
        ] {
            assert_eq!(strip_title(raw).venue.as_deref(), Some(want), "{raw}");
        }
    }

    #[test]
    fn a_live_segment_without_a_preposition_names_no_place() {
        // The guard, and the only thing standing between this and a map covered in venues called
        // "Session". A residue is not a location just because the recording was live.
        for raw in [
            "Song (Live Session)",
            "Song (Live Rehearsal)",
            "Song (Live Bootleg)",
        ] {
            assert_eq!(strip_title(raw).venue, None, "{raw}");
            // Still a live recording, and still kept in the title, which is what the residue rule
            // is for — this test is about the venue, not about refusing to parse anything.
            assert_eq!(strip_title(raw).variants, vec![TrackVariant::Live], "{raw}");
            assert_eq!(strip_title(raw).title, raw, "{raw}");
        }
    }

    #[test]
    fn a_segment_that_says_two_things_gets_two_badges() {
        // A recording can be more than one thing at once, and reporting only the first match hides
        // the track from a filter for everything else it is. "Live Acoustic" is the case that
        // prompted this: it used to come back acoustic, so a live-recordings list simply did not
        // contain it.
        for (raw, want) in [
            (
                "Song (Live Acoustic)",
                vec![TrackVariant::Live, TrackVariant::Acoustic],
            ),
            (
                "Song (Acoustic Demo)",
                vec![TrackVariant::Acoustic, TrackVariant::Demo],
            ),
            (
                "Song (Live Remastered)",
                vec![TrackVariant::Live, TrackVariant::Remaster],
            ),
            (
                "Song (Live Instrumental)",
                vec![TrackVariant::Live, TrackVariant::Instrumental],
            ),
        ] {
            let r = strip_title(raw);
            // Ordered by how much the marker changes the recording, not by tag order.
            assert_eq!(r.variants, want, "{raw}");
            // Both words are vocabulary, so the segment still subtracts to nothing and goes.
            assert_eq!(r.title, "Song", "{raw}");
        }
    }

    #[test]
    fn only_one_marker_answers_which_edit_this_is() {
        // The other half of the rule. Radio edit, single version, remix and extended are
        // alternatives, and they overlap on the same words — a "Radio Mix" is a radio edit, and
        // calling it a remix as well would be two badges saying one thing, one of them wrong.
        for (raw, want) in [
            ("Song (Radio Mix)", TrackVariant::RadioEdit),
            ("Song (Radio Edit)", TrackVariant::RadioEdit),
            ("Song (Single Version)", TrackVariant::SingleVersion),
            ("Song (Extended Mix)", TrackVariant::Remix),
            ("Song (Extended Version)", TrackVariant::Extended),
        ] {
            assert_eq!(strip_title(raw).variants, vec![want], "{raw}");
        }
        // But an independent marker still rides along with whichever one won.
        assert_eq!(
            strip_title("Song (Live Radio Edit)").variants,
            vec![TrackVariant::Live, TrackVariant::RadioEdit]
        );
    }

    #[test]
    fn a_name_that_begins_with_a_marker_word_survives() {
        // `residue` trims vocabulary off BOTH ends, which is right for deciding whether a segment
        // can be deleted and wrong for reading a name out of it: plenty of artists are called
        // something this file also treats as vocabulary. Crediting "Bandit" and "Killers" would be
        // worse than crediting nobody.
        assert_eq!(
            remixer("Song (Clean Bandit Remix)").as_deref(),
            Some("Clean Bandit")
        );
        assert_eq!(
            remixer("Song (The Killers Remix)").as_deref(),
            Some("The Killers")
        );
        // And "Clean" here is part of a band's name, not a content rating.
        assert_eq!(strip_title("Song (Clean Bandit Remix)").advisory, None);
        // A rating still reads when the segment really is only the rating.
        assert_eq!(strip_title("Song (Clean)").advisory, Some("clean"));
    }

    #[test]
    fn a_year_is_not_read_out_of_a_venue_name() {
        // "Wembley Arena 1985" and "Wembley Arena 1986" have to be one place, or grouping by venue
        // counts every night as somewhere new.
        let a = strip_title("Song (Live at Wembley Arena 1985)");
        let b = strip_title("Song (Live at Wembley Arena, 1986)");
        assert_eq!(a.venue, b.venue);
        assert_eq!(a.recorded_year, Some(1985));
        assert_eq!(b.recorded_year, Some(1986));
    }

    #[test]
    fn a_rating_still_reads_from_a_segment_that_stays() {
        // The advisory is a property of the recording, not of the strip, so it must survive a
        // segment that is kept for other reasons.
        let r = strip_title("Song (Gianni Marino Remix) [Explicit]");
        assert_eq!(r.title, "Song (Gianni Marino Remix)");
        assert_eq!(r.advisory, Some("explicit"));
        assert_eq!(r.remixer.as_deref(), Some("Gianni Marino"));
    }

    #[test]
    fn a_residue_too_long_to_be_a_name_is_not_credited() {
        // Bad rips glue a whole tracklist into one title. The badge still reads; the credit does
        // not, because a permanent junk artist is much worse than a missing one.
        assert_eq!(
            remixer("Song (Remix ft the whole crew live from the studio tonight)"),
            None
        );
    }
}
