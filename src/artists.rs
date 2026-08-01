//! Parsing combined artist strings into individual artists.
//!
//! A track's artist tag is often a single string crediting several people, such as
//! `"Drake feat. Rihanna"`, `"Calvin Harris feat. Dua Lipa & Young Thug"`, or a `"; "`-joined list
//! (how we serialise multi-value tags). The Hub turns each individual into its own enrichable
//! artist profile, and the library uses the primary (first) artist for on-disk placement.
//!
//! This lives in `contracts` so the Hub and the library split identically.
//!
//! ## Policy ("moderate")
//! - Split on `;` (independent credits / serialised multi-value tags).
//! - Within a credit, split on `feat.` / `ft.` / `featuring`; the featured remainder is further
//!   split on `&`, `,` and `/`.
//! - The primary (pre-`feat`) part is split on `,`, ` & ` and `/` (all three appear between two
//!   credited people — `"Mac Miller, Phonte"`, `"MGK/Cassie"`), with three guards so real names
//!   survive:
//!   1. a segment led by an article or connective is treated as a continuation of the previous name
//!      (so `"Tyler, The Creator"` / `"Florence & the Machine"` stay whole),
//!   2. a slash between two short tokens belongs to the name, not between names (so `"AC/DC"` and
//!      `"S/T"` stay whole) — see [`slash_separator`], and
//!   3. a small list of well-known band names ([`BAND_EXCEPTIONS`]) is never split.
//!
//! String heuristics can't be perfect. `"Vince Staples & Larry Fisherman"` (two artists) and
//! `"Simon & Garfunkel"` (one duo) are indistinguishable from the text alone. The authoritative
//! fix is MusicBrainz artist-credits (each contributor is its own entity with an explicit join
//! phrase), so this splitter is the fallback for un-enriched tracks, and the metadata override lets
//! a user correct any miss.

/// Well-known single-artist names that legitimately contain `,` or `&`; never split these.
/// (Slash-bearing names like `"AC/DC"` need no entry — [`slash_separator`] handles them by shape.)
/// Compared case-insensitively against the trimmed primary string. (Names of the form
/// `"X & The Y"` / `"X, The Y"` don't need listing because the continuation guard keeps them whole;
/// only `"X & Y"` / `"X, Y"` duos that are actually one act need an entry.)
const BAND_EXCEPTIONS: &[&str] = &[
    // comma-bearing
    "earth, wind & fire",
    "crosby, stills & nash",
    "crosby, stills, nash & young",
    "blood, sweat & tears",
    "emerson, lake & palmer",
    "peter, paul and mary",
    // ampersand-bearing duos/bands
    "simon & garfunkel",
    "hall & oates",
    "mumford & sons",
    "above & beyond",
    "sam & dave",
    "ike & tina turner",
    "loggins & messina",
    "brooks & dunn",
    "tegan & sara",
    "macklemore & ryan lewis",
];

/// Trim whitespace and stray bracket characters from one parsed name.
fn clean_name(s: &str) -> String {
    s.trim()
        .trim_matches(|c| matches!(c, '(' | ')' | '[' | ']'))
        .trim()
        .to_string()
}

/// Split a primary credit into individual artist names on `,`, ` & ` and `/`. A segment led by an
/// article or conjunction (e.g. `", The Creator"`, `" & the Machine"`) is re-joined to the previous
/// name as a continuation, names in [`BAND_EXCEPTIONS`] are returned whole, `&` without surrounding
/// spaces (e.g. `"R&B"`) is never a split point, and a slash inside a name (`"AC/DC"`) is left alone
/// by [`slash_separator`].
///
/// The continuation test used to be "does this segment start with a lowercase letter", which decided
/// artist IDENTITY from TYPOGRAPHY and got it wrong in both directions. Artists who style their names
/// in lower case were welded onto whoever preceded them — `"mgk, phem"` came through as the single
/// artist `"mgk, phem"`, and `"will.i.am, apl.de.ap"` likewise — while `"Tyler, The Creator"` was torn
/// into two artists because its `The` is capitalised. That second case is the very example the old
/// doc comment gave, written as `", the Creator"`; the real name capitalises it.
///
/// Matching a leading article instead is what was always meant. It is case-insensitive, so
/// `"Tyler, The Creator"` holds together, and it says nothing about the casing of a real name, so
/// `"phem"` and `"apl.de.ap"` are correctly their own artists.
fn split_primary(primary: &str) -> Vec<String> {
    let trimmed = primary.trim();
    if BAND_EXCEPTIONS.contains(&trimmed.to_ascii_lowercase().as_str()) {
        return vec![clean_name(trimmed)];
    }

    // Tokenize into (separator-before, segment) pairs, splitting on the earliest of `,`, ` & `, `/`.
    let mut segments: Vec<(&str, &str)> = Vec::new();
    let mut sep_before = "";
    let mut rest = trimmed;
    loop {
        // Each candidate is (byte offset, separator length, canonical form used to rejoin a
        // continuation). The earliest one wins.
        let candidates = [
            rest.find(',').map(|i| (i, 1, ", ")),
            rest.find(" & ").map(|i| (i, 3, " & ")),
            slash_separator(rest).map(|i| (i, 1, "/")),
        ];
        let Some((cut, sep_len, sep)) = candidates.into_iter().flatten().min_by_key(|(i, ..)| *i)
        else {
            segments.push((sep_before, rest));
            break;
        };
        segments.push((sep_before, &rest[..cut]));
        sep_before = sep;
        rest = &rest[cut + sep_len..];
    }

    // An article-led segment continues the previous name (rejoined with its original separator).
    let mut out: Vec<String> = Vec::new();
    for (sep, seg) in segments {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        if is_continuation(seg) && !out.is_empty() {
            let last = out.last_mut().expect("non-empty");
            *last = format!("{last}{sep}{seg}");
        } else {
            out.push(seg.to_string());
        }
    }
    out.iter()
        .map(|p| clean_name(p))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Byte offset of the first `/` that separates two artists, or `None` if every slash in `s` belongs
/// inside a name.
///
/// ID3 has used `/` as a multi-value separator since v2.2, so taggers emit `"MGK/Cassie"` and
/// `"MGK/Bun B/Dub-o"` meaning several credited artists — and treating those as one name minted a
/// combined artist row per collaborator, about twenty of them from a single library.
///
/// The one name that must survive is `AC/DC`, and the spacing trick used for `&` (require ` & `, so
/// `R&B` is safe) is no help here: these tags have no spaces around the slash either. What does
/// separate them is length. An initialism split by a slash is short on both sides — `AC`/`DC`,
/// `S/T` — where a real credit has at least one substantial side. So a slash flanked by two tokens
/// of at most two characters is part of the name; anything else divides two artists. That is
/// structural rather than a name list, so it also holds when `AC/DC` appears mid-string, which a
/// whole-string exception could not do.
fn slash_separator(s: &str) -> Option<usize> {
    s.char_indices()
        .filter(|(_, c)| *c == '/')
        .map(|(i, _)| i)
        .find(|&i| {
            // The adjacent tokens, bounded by spaces and other slashes.
            let before = s[..i].rsplit([' ', '/']).next().unwrap_or("");
            let after = s[i + 1..].split([' ', '/']).next().unwrap_or("");
            if before.is_empty() || after.is_empty() {
                return false; // a dangling slash separates nothing
            }
            let short = |t: &str| t.chars().count() <= 2;
            !(short(before) && short(after))
        })
}

/// Split one name on its genuine artist-separating slashes (see [`slash_separator`]). Returns the
/// input untouched when it holds none, so `"AC/DC"` stays whole.
fn slash_parts(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(i) = slash_separator(rest) {
        out.push(&rest[..i]);
        rest = &rest[i + 1..];
    }
    out.push(rest);
    out
}

/// Leading words that make a segment a CONTINUATION of the name before it rather than a new artist:
/// `"Tyler, The Creator"`, `"Florence & the Machine"`, `"Bob Marley & the Wailers"`. These are
/// articles and connectives, never an artist name on their own, so a segment starting with one
/// cannot be a separate credit. Deliberately small — every entry is a word that reads as a fragment.
const CONTINUATION_LEADERS: &[&str] = &[
    "the", "a", "an", "his", "her", "their", "its", "los", "las", "les", "la", "le", "el", "die",
    "der", "das", "il", "de", "het", "os", "as",
];

/// Does `segment` continue the previous name instead of naming a new artist?
fn is_continuation(segment: &str) -> bool {
    segment
        .split_whitespace()
        .next()
        .map(|w| {
            let w: String = w
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect();
            // A bare leader with nothing after it ("Earth, The") is not a continuation of anything
            // useful; require the segment to carry a real word too.
            !w.is_empty()
                && CONTINUATION_LEADERS.contains(&w.as_str())
                && segment.split_whitespace().count() > 1
        })
        .unwrap_or(false)
}

/// Is `token` a "featuring" marker (allowing a leading `(` and trailing `.`/`)`)?
fn is_feat_marker(token: &str) -> bool {
    let t = token
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim_end_matches('.')
        .to_ascii_lowercase();
    matches!(t.as_str(), "feat" | "ft" | "featuring")
}

/// Split one credit into all its artist names, primary (pre-`feat`) names first then featured.
fn split_credit(credit: &str) -> Vec<String> {
    let tokens: Vec<&str> = credit.split_whitespace().collect();
    let (primary_str, feat_str) = match tokens.iter().position(|t| is_feat_marker(t)) {
        Some(pos) => (tokens[..pos].join(" "), Some(tokens[pos + 1..].join(" "))),
        None => (credit.to_string(), None),
    };
    let mut names = split_primary(&primary_str);
    if let Some(featured) = feat_str {
        names.extend(
            featured
                .split(['&', ','])
                // Same slash rule as the primary half, so `"feat. MGK/Cassie"` is two artists while
                // `"feat. AC/DC"` stays one.
                .flat_map(slash_parts)
                .map(clean_name)
                .filter(|s| !s.is_empty()),
        );
    }
    names
}

/// Split a combined artist string into individual artist names, ordered with the primary first and
/// duplicates removed (case-insensitively). Returns an empty vec only for an empty/whitespace input.
pub fn split_artists(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |name: String| {
        if !name.is_empty() && !out.iter().any(|e| e.eq_ignore_ascii_case(&name)) {
            out.push(name);
        }
    };
    for credit in s.split(';') {
        for name in split_credit(credit) {
            push(name);
        }
    }
    out
}

/// The primary (first) artist of a combined string, falling back to the trimmed input.
pub fn primary_artist(s: &str) -> String {
    split_artists(s)
        .into_iter()
        .next()
        .unwrap_or_else(|| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_artist_unchanged() {
        assert_eq!(split_artists("Pink Floyd"), vec!["Pink Floyd"]);
    }

    #[test]
    fn semicolon_joined_splits() {
        assert_eq!(split_artists("Drake; Rihanna"), vec!["Drake", "Rihanna"]);
    }

    #[test]
    fn feat_splits_and_keeps_primary_whole() {
        assert_eq!(
            split_artists("Drake feat. Rihanna"),
            vec!["Drake", "Rihanna"]
        );
        // Primary with an ampersand stays intact; only the featured part splits.
        assert_eq!(
            split_artists("Calvin Harris feat. Dua Lipa & Young Thug"),
            vec!["Calvin Harris", "Dua Lipa", "Young Thug"]
        );
    }

    #[test]
    fn parenthetical_feat() {
        assert_eq!(
            split_artists("Drake (feat. Rihanna)"),
            vec!["Drake", "Rihanna"]
        );
    }

    #[test]
    fn ampersand_collaboration_splits() {
        // The reported case: "X & Y" collaboration becomes two artists.
        assert_eq!(
            split_artists("Vince Staples & Larry Fisherman"),
            vec!["Vince Staples", "Larry Fisherman"]
        );
        assert_eq!(
            primary_artist("Vince Staples & Larry Fisherman"),
            "Vince Staples"
        );
    }

    /// A collaborator styling their name in lower case is still a separate artist. The old rule read
    /// a leading lowercase letter as "this continues the previous name", so `"mgk, phem"` reached the
    /// Hub as one artist literally called `"mgk, phem"` and got its own row and its own two tracks.
    #[test]
    fn lowercase_styled_collaborators_still_split() {
        assert_eq!(split_artists("mgk, phem"), vec!["mgk", "phem"]);
        assert_eq!(split_artists("mgk & phem"), vec!["mgk", "phem"]);
        assert_eq!(
            split_artists("will.i.am, apl.de.ap"),
            vec!["will.i.am", "apl.de.ap"]
        );
        assert_eq!(split_artists("aespa, NCT"), vec!["aespa", "NCT"]);
        assert_eq!(primary_artist("mgk, phem"), "mgk");
    }

    /// ID3 has used `/` as a multi-value separator since v2.2, so a slash between credits is a real
    /// artist boundary. Not splitting on it minted a combined row per collaborator — "MGK/Cassie",
    /// "MGK/DMX", "MGK/Bun B/Dub-o" and about twenty more from one library.
    #[test]
    fn slash_separates_credited_artists() {
        assert_eq!(split_artists("MGK/Cassie"), vec!["MGK", "Cassie"]);
        assert_eq!(split_artists("MGK/DMX"), vec!["MGK", "DMX"]);
        assert_eq!(
            split_artists("MGK/Bun B/Dub-o"),
            vec!["MGK", "Bun B", "Dub-o"]
        );
        assert_eq!(
            split_artists("Miles Davis/John Coltrane"),
            vec!["Miles Davis", "John Coltrane"]
        );
        assert_eq!(
            split_artists("Drake feat. MGK/Cassie"),
            vec!["Drake", "MGK", "Cassie"]
        );
        assert_eq!(primary_artist("MGK/Alex Fritts"), "MGK");
    }

    /// The counterweight: a slash flanked by two short tokens is inside the name. `&` is protected by
    /// requiring spaces around it, which is no help here — these tags have no spaces either way — so
    /// the guard keys on length instead. It must hold wherever the name appears.
    #[test]
    fn slash_inside_a_name_is_not_a_separator() {
        assert_eq!(split_artists("AC/DC"), vec!["AC/DC"]);
        assert_eq!(split_artists("S/T"), vec!["S/T"]);
        assert_eq!(
            split_artists("AC/DC & Guns N' Roses"),
            vec!["AC/DC", "Guns N' Roses"]
        );
        assert_eq!(split_artists("Drake feat. AC/DC"), vec!["Drake", "AC/DC"]);
        assert_eq!(primary_artist("AC/DC"), "AC/DC");
    }

    /// The mirror of the above, and the case the old rule broke in the other direction: a capitalised
    /// article is still an article, so this is ONE artist. The old doc comment cited this very name
    /// but spelled it `", the Creator"` — with the real capital `T`, the splitter tore it in half.
    #[test]
    fn capitalised_article_holds_the_name_together() {
        assert_eq!(
            split_artists("Tyler, The Creator"),
            vec!["Tyler, The Creator"]
        );
        assert_eq!(
            split_artists("Tyler, the Creator"),
            vec!["Tyler, the Creator"]
        );
        assert_eq!(primary_artist("Tyler, The Creator"), "Tyler, The Creator");
        for band in [
            "Florence & the Machine",
            "Bob Marley & the Wailers",
            "Nick Cave & the Bad Seeds",
            "Selena Gomez & the Scene",
            "Huey Lewis & the News",
        ] {
            assert_eq!(split_artists(band), vec![band], "{band} must stay whole");
        }
    }

    #[test]
    fn ampersand_band_names_preserved() {
        // "X & the Y": the lowercase continuation keeps it whole (no list needed).
        assert_eq!(
            split_artists("Florence & the Machine"),
            vec!["Florence & the Machine"]
        );
        // Known "X & Y" duos that are actually one act: the exception list keeps them whole.
        assert_eq!(
            split_artists("Simon & Garfunkel"),
            vec!["Simon & Garfunkel"]
        );
        assert_eq!(split_artists("Hall & Oates"), vec!["Hall & Oates"]);
        // `&` without spaces is not a separator.
        assert_eq!(split_artists("R&B Allstars"), vec!["R&B Allstars"]);
    }

    #[test]
    fn comma_collaboration_splits() {
        // The reported bug: "Mac Miller, Phonte" must become two artists, primary first.
        assert_eq!(
            split_artists("Mac Miller, Phonte"),
            vec!["Mac Miller", "Phonte"]
        );
        assert_eq!(primary_artist("Mac Miller, Phonte"), "Mac Miller");
        assert_eq!(split_artists("A, B, C"), vec!["A", "B", "C"]);
    }

    #[test]
    fn comma_band_names_preserved() {
        // Lowercase continuation, so it stays whole.
        assert_eq!(
            split_artists("Tyler, the Creator"),
            vec!["Tyler, the Creator"]
        );
        // Known exceptions stay whole (the `&` part isn't split either).
        assert_eq!(
            split_artists("Earth, Wind & Fire"),
            vec!["Earth, Wind & Fire"]
        );
        // A comma collaboration whose first member is a comma-band still splits on the outer commas
        // but keeps the lowercase continuation attached.
        assert_eq!(
            split_artists("Tyler, the Creator, Frank Ocean"),
            vec!["Tyler, the Creator", "Frank Ocean"]
        );
    }

    #[test]
    fn dedupes_case_insensitively() {
        assert_eq!(split_artists("Drake; drake feat. Drake"), vec!["Drake"]);
    }

    #[test]
    fn primary_is_first() {
        assert_eq!(primary_artist("Drake feat. Rihanna"), "Drake");
        assert_eq!(primary_artist("A; B; C"), "A");
        assert_eq!(primary_artist("  Solo  "), "Solo");
    }
}
