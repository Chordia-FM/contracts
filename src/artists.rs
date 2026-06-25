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
//!   split on `&` and `,`.
//! - The primary (pre-`feat`) part is split on commas (a comma between two credited people is
//!   the common case, e.g. `"Mac Miller, Phonte"`), with two guards so band names survive:
//!   1. a segment beginning with a lowercase word is treated as a continuation of the previous name
//!      (so `"Tyler, the Creator"` / `"Florence & the Machine"` stay whole), and
//!   2. a small list of well-known band names ([`BAND_EXCEPTIONS`]) is never split.
//!
//! String heuristics can't be perfect. `"Vince Staples & Larry Fisherman"` (two artists) and
//! `"Simon & Garfunkel"` (one duo) are indistinguishable from the text alone. The authoritative
//! fix is MusicBrainz artist-credits (each contributor is its own entity with an explicit join
//! phrase), so this splitter is the fallback for un-enriched tracks, and the metadata override lets
//! a user correct any miss.

/// Well-known single-artist names that legitimately contain `,` or `&`; never split these.
/// Compared case-insensitively against the trimmed primary string. (Names of the form
/// `"X & the Y"` / `"X, the Y"` don't need listing because the lowercase-continuation guard keeps
/// them whole; only `"X & Y"` / `"X, Y"` duos that are actually one act need an entry.)
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

/// Split a primary credit into individual artist names on `,` and ` & `. A segment whose first
/// word is lowercase (e.g. `", the Creator"`, `" & the Machine"`) is re-joined to the previous name
/// (with its original separator) as a continuation, and names in [`BAND_EXCEPTIONS`] are returned
/// whole. `&` without surrounding spaces (e.g. `"R&B"`) is never a split point.
fn split_primary(primary: &str) -> Vec<String> {
    let trimmed = primary.trim();
    if BAND_EXCEPTIONS.contains(&trimmed.to_ascii_lowercase().as_str()) {
        return vec![clean_name(trimmed)];
    }

    // Tokenize into (separator-before, segment) pairs, splitting on the earliest of `,` or ` & `.
    let mut segments: Vec<(&str, &str)> = Vec::new();
    let mut sep_before = "";
    let mut rest = trimmed;
    loop {
        let comma = rest.find(',');
        let amp = rest.find(" & ");
        // `cut`/`sep_len` index the source; `sep` is the canonical form used to rejoin continuations.
        let (cut, sep_len, sep) = match (comma, amp) {
            (Some(c), Some(a)) if c < a => (c, 1, ", "),
            (Some(_), Some(a)) => (a, 3, " & "),
            (Some(c), None) => (c, 1, ", "),
            (None, Some(a)) => (a, 3, " & "),
            (None, None) => {
                segments.push((sep_before, rest));
                break;
            }
        };
        segments.push((sep_before, &rest[..cut]));
        sep_before = sep;
        rest = &rest[cut + sep_len..];
    }

    // A lowercase-led segment continues the previous name (rejoined with its original separator).
    let mut out: Vec<String> = Vec::new();
    for (sep, seg) in segments {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        let starts_lower = seg
            .chars()
            .next()
            .map(|c| c.is_lowercase())
            .unwrap_or(false);
        if starts_lower && !out.is_empty() {
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
