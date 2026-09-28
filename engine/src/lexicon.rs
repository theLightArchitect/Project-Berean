//! Original-language lookups: Strong's number, morphology, gloss, and full
//! definition. Same contract as corpus.rs: say "not found", don't guess.

use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::reference::normalize_strongs;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct LexiconQuery {
    /// A Strong's number (e.g. "G26", "H0430"). Looking up by the word
    /// itself (rather than its Strong's number) isn't supported yet — see
    /// the note on `found` below.
    pub word_or_strongs: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LexiconResult {
    pub query: String,
    pub strongs_number: Option<String>,
    pub lemma: Option<String>,
    pub transliteration: Option<String>,
    pub morphology: Option<String>,
    pub gloss: Option<String>,
    pub definition: Option<String>,
    pub found: bool,
}

/// Looks up a Strong's number against the corpus database's lexicon table
/// (BDB for Hebrew, Abbott-Smith-derived for Greek — see
/// `docs/TOOL-PALETTE.md`). If `word_or_strongs` isn't a recognizable
/// Strong's number, or the database isn't loaded, or the number isn't in
/// the lexicon: `found: false`. This never falls back to a guessed
/// definition — a word-based (rather than Strong's-number-based) lookup
/// path is real future work, not a fuzzy match hiding behind this contract.
pub fn lookup_lexicon(query: LexiconQuery, conn: Option<&Connection>) -> LexiconResult {
    let not_found = || LexiconResult {
        query: query.word_or_strongs.clone(),
        strongs_number: None,
        lemma: None,
        transliteration: None,
        morphology: None,
        gloss: None,
        definition: None,
        found: false,
    };

    let Some(conn) = conn else {
        return not_found();
    };
    let Some(canonical) = normalize_strongs(&query.word_or_strongs) else {
        return not_found();
    };

    let row = conn.query_row(
        "SELECT lemma, transliteration, morphology, gloss, definition FROM lexicon WHERE strongs = ?1",
        [&canonical],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        },
    );

    match row {
        Ok((lemma, transliteration, morphology, gloss, definition)) => LexiconResult {
            query: query.word_or_strongs,
            strongs_number: Some(canonical),
            lemma: Some(lemma),
            transliteration,
            morphology,
            gloss,
            definition,
            found: true,
        },
        Err(_) => not_found(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_fabricates_a_gloss_for_an_unloaded_lexicon() {
        let result = lookup_lexicon(
            LexiconQuery {
                word_or_strongs: "G26".into(),
            },
            None,
        );
        assert!(!result.found);
        assert!(result.gloss.is_none());
    }

    #[test]
    fn rejects_a_non_strongs_query_rather_than_guess() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE lexicon (strongs TEXT PRIMARY KEY, language TEXT, lemma TEXT, transliteration TEXT, morphology TEXT, gloss TEXT, definition TEXT);",
        )
        .unwrap();
        let result = lookup_lexicon(
            LexiconQuery {
                word_or_strongs: "love".into(),
            },
            Some(&conn),
        );
        assert!(!result.found);
    }

    #[test]
    fn finds_a_real_entry_and_normalizes_padded_strongs_numbers() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE lexicon (strongs TEXT PRIMARY KEY, language TEXT, lemma TEXT, transliteration TEXT, morphology TEXT, gloss TEXT, definition TEXT);
             INSERT INTO lexicon VALUES ('G26', 'greek', 'ἀγάπη', 'agape', 'N-F', 'love', 'brotherly love, affection, good will, love feasts');",
        )
        .unwrap();

        let result = lookup_lexicon(
            LexiconQuery {
                word_or_strongs: "G0026".into(),
            },
            Some(&conn),
        );
        assert!(result.found);
        assert_eq!(result.strongs_number.as_deref(), Some("G26"));
        assert_eq!(result.gloss.as_deref(), Some("love"));
        assert_eq!(result.lemma.as_deref(), Some("ἀγάπη"));
    }
}
