//! Scripture reference parsing: turns a human string like "John 3:16" or
//! "Genesis 1:1-5" into the book code Berean's corpus is keyed by (the
//! 3-letter codes used throughout the ingested BSB dataset — GEN, EXO, ...,
//! REV) plus a chapter and verse range.
//!
//! This is foundational, not incidental: every lookup tool depends on
//! parsing the reference correctly, and a silent mis-parse (mapping "Judges"
//! to the wrong book, say) would be exactly the kind of quiet failure the
//! rest of the engine exists to prevent. So parsing is strict: anything
//! ambiguous or unsupported (cross-chapter ranges, unknown books) is a
//! parse error, never a best-effort guess.

/// (code, full name, aliases). Codes and full names match the 66-book set
/// used by the ingested dataset exactly (see `ls base/display/` in the
/// source data) — this list is the single source of truth for book
/// identity in the engine.
const BOOKS: &[(&str, &str, &[&str])] = &[
    ("GEN", "Genesis", &["gen"]),
    ("EXO", "Exodus", &["ex", "exo"]),
    ("LEV", "Leviticus", &["lev"]),
    ("NUM", "Numbers", &["num", "nu"]),
    ("DEU", "Deuteronomy", &["deut", "dt"]),
    ("JOS", "Joshua", &["josh"]),
    ("JDG", "Judges", &["judg", "jdgs"]),
    ("RUT", "Ruth", &["rut"]),
    ("1SA", "1 Samuel", &["1sam", "1st samuel", "i samuel"]),
    ("2SA", "2 Samuel", &["2sam", "2nd samuel", "ii samuel"]),
    ("1KI", "1 Kings", &["1kgs", "1st kings", "i kings"]),
    ("2KI", "2 Kings", &["2kgs", "2nd kings", "ii kings"]),
    (
        "1CH",
        "1 Chronicles",
        &["1chr", "1st chronicles", "i chronicles"],
    ),
    (
        "2CH",
        "2 Chronicles",
        &["2chr", "2nd chronicles", "ii chronicles"],
    ),
    ("EZR", "Ezra", &["ezr"]),
    ("NEH", "Nehemiah", &["neh"]),
    ("EST", "Esther", &["esth"]),
    ("JOB", "Job", &["job"]),
    ("PSA", "Psalm", &["psalms", "ps", "pslm"]),
    ("PRO", "Proverbs", &["prov", "pr"]),
    ("ECC", "Ecclesiastes", &["eccl", "eccles"]),
    (
        "SNG",
        "Song of Solomon",
        &["song of songs", "sos", "canticles"],
    ),
    ("ISA", "Isaiah", &["isa"]),
    ("JER", "Jeremiah", &["jer"]),
    ("LAM", "Lamentations", &["lam"]),
    ("EZK", "Ezekiel", &["ezek", "eze"]),
    ("DAN", "Daniel", &["dan"]),
    ("HOS", "Hosea", &["hos"]),
    ("JOL", "Joel", &["joe"]),
    ("AMO", "Amos", &["amo"]),
    ("OBA", "Obadiah", &["obad"]),
    ("JON", "Jonah", &["jnh"]),
    ("MIC", "Micah", &["mic"]),
    ("NAM", "Nahum", &["nah"]),
    ("HAB", "Habakkuk", &["hab"]),
    ("ZEP", "Zephaniah", &["zeph"]),
    ("HAG", "Haggai", &["hag"]),
    ("ZEC", "Zechariah", &["zech"]),
    ("MAL", "Malachi", &["mal"]),
    ("MAT", "Matthew", &["matt", "mt"]),
    ("MRK", "Mark", &["mk"]),
    ("LUK", "Luke", &["lk"]),
    ("JHN", "John", &["jn", "jhn"]),
    ("ACT", "Acts", &["act"]),
    ("ROM", "Romans", &["rom"]),
    (
        "1CO",
        "1 Corinthians",
        &["1cor", "1st corinthians", "i corinthians"],
    ),
    (
        "2CO",
        "2 Corinthians",
        &["2cor", "2nd corinthians", "ii corinthians"],
    ),
    ("GAL", "Galatians", &["gal"]),
    ("EPH", "Ephesians", &["eph"]),
    ("PHP", "Philippians", &["phil"]),
    ("COL", "Colossians", &["col"]),
    (
        "1TH",
        "1 Thessalonians",
        &["1thess", "1st thessalonians", "i thessalonians"],
    ),
    (
        "2TH",
        "2 Thessalonians",
        &["2thess", "2nd thessalonians", "ii thessalonians"],
    ),
    ("1TI", "1 Timothy", &["1tim", "1st timothy", "i timothy"]),
    ("2TI", "2 Timothy", &["2tim", "2nd timothy", "ii timothy"]),
    ("TIT", "Titus", &["tit"]),
    ("PHM", "Philemon", &["philem"]),
    ("HEB", "Hebrews", &["heb"]),
    ("JAS", "James", &["jas"]),
    ("1PE", "1 Peter", &["1pet", "1st peter", "i peter"]),
    ("2PE", "2 Peter", &["2pet", "2nd peter", "ii peter"]),
    ("1JN", "1 John", &["1john", "1st john", "i john"]),
    ("2JN", "2 John", &["2john", "2nd john", "ii john"]),
    ("3JN", "3 John", &["3john", "3rd john", "iii john"]),
    ("JUD", "Jude", &["jde"]),
    ("REV", "Revelation", &["rev", "revelations"]),
];

/// A single-chapter reference, possibly a verse range: e.g. John 3:16-18 is
/// `{ book: "JHN", chapter: 3, verse_start: 16, verse_end: 18 }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub book: String,
    pub chapter: u32,
    pub verse_start: u32,
    pub verse_end: u32,
}

impl Reference {
    pub fn verses(&self) -> impl Iterator<Item = u32> {
        self.verse_start..=self.verse_end
    }
}

fn normalize(s: &str) -> String {
    s.trim().to_lowercase().replace('.', "").replace("  ", " ")
}

/// All 66 three-letter book codes, in canonical (Protestant) order. Used by
/// the ingest binary to match dataset filenames against real book
/// identities rather than a guessed-at prefix rule.
pub fn book_codes() -> impl Iterator<Item = &'static str> {
    BOOKS.iter().map(|(code, _, _)| *code)
}

/// Canonicalize a Strong's number: same letter, digits with leading zeros
/// stripped (so "H0001", "H1", and "h1" all normalize to "H1"). This is the
/// key both lexicon ingestion and lookup use, so a lookup for "G0026" finds
/// the same row ingested from "G26" or vice versa.
pub fn normalize_strongs(input: &str) -> Option<String> {
    let input = input.trim();
    let mut chars = input.chars();
    let letter = chars.next()?.to_ascii_uppercase();
    if letter != 'G' && letter != 'H' {
        return None;
    }
    let digits: String = chars.collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let number: u32 = digits.parse().ok()?;
    Some(format!("{letter}{number}"))
}

/// Resolve a book name/abbreviation (any case, with or without punctuation)
/// to its 3-letter code. Returns None for anything not recognized — never
/// guesses at the nearest match.
pub fn resolve_book(input: &str) -> Option<&'static str> {
    let normalized = normalize(input);
    for (code, full_name, aliases) in BOOKS {
        if normalized == code.to_lowercase() || normalized == full_name.to_lowercase() {
            return Some(code);
        }
        if aliases.iter().any(|a| *a == normalized) {
            return Some(code);
        }
    }
    None
}

/// Parse "Book C:V" or "Book C:V-V2". Cross-chapter ranges ("John 1:1-2:5")
/// and whole-chapter references ("John 3") are not supported — return None
/// rather than guess which verses were meant.
pub fn parse(input: &str) -> Option<Reference> {
    let input = input.trim();
    let colon_pos = input.rfind(':')?;
    let (book_and_chapter, verse_part) = input.split_at(colon_pos);
    let verse_part = &verse_part[1..]; // drop the ':'

    let last_space = book_and_chapter.rfind(' ')?;
    let (book_part, chapter_part) = book_and_chapter.split_at(last_space);
    let chapter_part = chapter_part.trim();

    let book = resolve_book(book_part)?;
    let chapter: u32 = chapter_part.parse().ok()?;

    let (verse_start, verse_end) = if let Some((start, end)) = verse_part.split_once('-') {
        (start.trim().parse().ok()?, end.trim().parse().ok()?)
    } else {
        let v: u32 = verse_part.trim().parse().ok()?;
        (v, v)
    };

    if verse_end < verse_start {
        return None;
    }

    Some(Reference {
        book: book.to_string(),
        chapter,
        verse_start,
        verse_end,
    })
}

/// The reverse of `resolve_book`: 3-letter code -> full display name.
pub fn book_full_name(code: &str) -> Option<&'static str> {
    BOOKS
        .iter()
        .find(|(c, _, _)| *c == code)
        .map(|(_, name, _)| *name)
}

/// Render a compact id like "JHN.3.16" back to a human-readable reference
/// string, e.g. "John 3:16". Used for cross-reference targets, which the
/// source data stores in compact form.
pub fn compact_id_to_display(compact: &str) -> Option<String> {
    let mut parts = compact.split('.');
    let code = parts.next()?;
    let chapter = parts.next()?;
    let verse = parts.next()?;
    let name = book_full_name(code)?;
    Some(format!("{name} {chapter}:{verse}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_strongs_numbers_regardless_of_padding_or_case() {
        assert_eq!(normalize_strongs("H0001").as_deref(), Some("H1"));
        assert_eq!(normalize_strongs("H1").as_deref(), Some("H1"));
        assert_eq!(normalize_strongs("g0026").as_deref(), Some("G26"));
    }

    #[test]
    fn rejects_a_malformed_strongs_number_rather_than_guess() {
        assert_eq!(normalize_strongs("X1"), None);
        assert_eq!(normalize_strongs("G"), None);
        assert_eq!(normalize_strongs("Grace"), None);
    }

    #[test]
    fn book_codes_covers_all_66_books() {
        assert_eq!(book_codes().count(), 66);
    }

    #[test]
    fn resolves_full_names_codes_and_aliases_case_insensitively() {
        assert_eq!(resolve_book("John"), Some("JHN"));
        assert_eq!(resolve_book("JHN"), Some("JHN"));
        assert_eq!(resolve_book("jhn"), Some("JHN"));
        assert_eq!(resolve_book("Jn"), Some("JHN"));
        assert_eq!(resolve_book("1 Corinthians"), Some("1CO"));
        assert_eq!(resolve_book("1cor"), Some("1CO"));
        assert_eq!(resolve_book("Song of Solomon"), Some("SNG"));
    }

    #[test]
    fn never_guesses_at_an_unrecognized_book() {
        assert_eq!(resolve_book("Gospel of Thomas"), None);
        assert_eq!(resolve_book(""), None);
    }

    #[test]
    fn parses_single_verse_and_verse_range() {
        assert_eq!(
            parse("John 3:16"),
            Some(Reference {
                book: "JHN".into(),
                chapter: 3,
                verse_start: 16,
                verse_end: 16,
            })
        );
        assert_eq!(
            parse("Genesis 1:1-5"),
            Some(Reference {
                book: "GEN".into(),
                chapter: 1,
                verse_start: 1,
                verse_end: 5,
            })
        );
    }

    #[test]
    fn rejects_cross_chapter_ranges_and_whole_chapter_refs_rather_than_guess() {
        assert_eq!(parse("John 1:1-2:5"), None);
        assert_eq!(parse("John 3"), None);
        assert_eq!(parse("not a reference"), None);
    }

    #[test]
    fn rejects_an_inverted_range() {
        assert_eq!(parse("John 3:18-16"), None);
    }

    #[test]
    fn compact_id_round_trips_to_a_readable_reference() {
        assert_eq!(
            compact_id_to_display("JHN.3.16"),
            Some("John 3:16".to_string())
        );
        assert_eq!(compact_id_to_display("NOPE.1.1"), None);
    }
}
