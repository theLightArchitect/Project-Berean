//! Word-by-word interlinear: the original-language words (in their own
//! reading order) and the English rendering (in its own reading order),
//! each tagged with Strong's numbers where applicable.
//!
//! Deliberately NOT zipped into one aligned word-for-word table: Hebrew and
//! Greek word order routinely differs from natural English syntax (e.g.
//! Genesis 1:1's Hebrew reads "In-beginning created God..."; the English
//! gloss reads "In the beginning God created..."). A positional pairing
//! between the two arrays would silently assert a specific word
//! correspondence the source data doesn't actually claim — exactly the kind
//! of fabrication this engine exists to avoid. Present both reading orders;
//! let the shared Strong's number be the only claimed link between them.

use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::reference;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct InterlinearQuery {
    /// e.g. "John 1:1-3". Only the BSB-aligned interlinear is available —
    /// there is no translation field because there is only one source.
    pub reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OriginalWord {
    /// The Hebrew or Greek word form, in original reading order.
    pub word: String,
    pub strongs: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnglishSegment {
    /// English rendering order. Includes spacer/punctuation segments
    /// (`strongs: None`) so the full sentence can be reconstructed.
    pub text: String,
    pub strongs: Option<String>,
    pub elided: bool,
    pub supplied: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterlinearVerse {
    pub verse: u32,
    pub original: Vec<OriginalWord>,
    pub english: Vec<EnglishSegment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterlinearResult {
    pub reference: String,
    pub verses: Vec<InterlinearVerse>,
    pub found: bool,
}

/// Same discipline as corpus.rs: a verse range is only returned if every
/// requested verse actually has data — never a silently incomplete range.
pub fn get_interlinear(query: InterlinearQuery, conn: Option<&Connection>) -> InterlinearResult {
    let not_found = || InterlinearResult {
        reference: query.reference.clone(),
        verses: Vec::new(),
        found: false,
    };

    let Some(conn) = conn else {
        return not_found();
    };
    let Some(parsed) = reference::parse(&query.reference) else {
        return not_found();
    };

    let mut original_stmt = match conn.prepare(
        "SELECT verse, word, strongs FROM interlinear_original
         WHERE book = ?1 AND chapter = ?2 AND verse BETWEEN ?3 AND ?4
         ORDER BY verse, seq",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return not_found(),
    };
    let mut english_stmt = match conn.prepare(
        "SELECT verse, text, strongs, elided, supplied FROM interlinear_english
         WHERE book = ?1 AND chapter = ?2 AND verse BETWEEN ?3 AND ?4
         ORDER BY verse, seq",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return not_found(),
    };

    let params = rusqlite::params![
        parsed.book,
        parsed.chapter,
        parsed.verse_start,
        parsed.verse_end
    ];

    let original_rows = original_stmt.query_map(params, |row| {
        Ok((
            row.get::<_, u32>(0)?,
            OriginalWord {
                word: row.get(1)?,
                strongs: row.get(2)?,
            },
        ))
    });
    let Ok(original_rows) = original_rows else {
        return not_found();
    };

    let english_rows = english_stmt.query_map(params, |row| {
        Ok((
            row.get::<_, u32>(0)?,
            EnglishSegment {
                text: row.get(1)?,
                strongs: row.get(2)?,
                elided: row.get::<_, i64>(3)? != 0,
                supplied: row.get::<_, i64>(4)? != 0,
            },
        ))
    });
    let Ok(english_rows) = english_rows else {
        return not_found();
    };

    let mut verses: std::collections::BTreeMap<u32, InterlinearVerse> =
        std::collections::BTreeMap::new();
    for row in original_rows.flatten() {
        let (verse, word) = row;
        verses
            .entry(verse)
            .or_insert_with(|| InterlinearVerse {
                verse,
                original: Vec::new(),
                english: Vec::new(),
            })
            .original
            .push(word);
    }
    for row in english_rows.flatten() {
        let (verse, segment) = row;
        verses
            .entry(verse)
            .or_insert_with(|| InterlinearVerse {
                verse,
                original: Vec::new(),
                english: Vec::new(),
            })
            .english
            .push(segment);
    }

    let expected = (parsed.verse_end - parsed.verse_start + 1) as usize;
    if verses.len() != expected
        || verses
            .values()
            .any(|v| v.original.is_empty() || v.english.is_empty())
    {
        return not_found();
    }

    InterlinearResult {
        reference: query.reference,
        verses: verses.into_values().collect(),
        found: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_fabricates_interlinear_data_for_an_unloaded_corpus() {
        let result = get_interlinear(
            InterlinearQuery {
                reference: "John 1:1".into(),
            },
            None,
        );
        assert!(!result.found);
        assert!(result.verses.is_empty());
    }

    fn seeded_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE interlinear_original (book TEXT, chapter INTEGER, verse INTEGER, seq INTEGER, word TEXT, strongs TEXT);
             CREATE TABLE interlinear_english (book TEXT, chapter INTEGER, verse INTEGER, seq INTEGER, text TEXT, strongs TEXT, elided INTEGER, supplied INTEGER);
             INSERT INTO interlinear_original VALUES ('GEN', 1, 1, 0, 'בְּרֵאשִׁ֖ית', 'H7225');
             INSERT INTO interlinear_original VALUES ('GEN', 1, 1, 1, 'בָּרָ֣א', 'H1254');
             INSERT INTO interlinear_english VALUES ('GEN', 1, 1, 0, 'In the beginning', 'H7225', 0, 0);
             INSERT INTO interlinear_english VALUES ('GEN', 1, 1, 1, ' ', NULL, 0, 0);
             INSERT INTO interlinear_english VALUES ('GEN', 1, 1, 2, 'God', 'H430', 0, 0);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn finds_real_interlinear_data_with_both_reading_orders_preserved() {
        let conn = seeded_connection();
        let result = get_interlinear(
            InterlinearQuery {
                reference: "Genesis 1:1".into(),
            },
            Some(&conn),
        );
        assert!(result.found);
        let verse = &result.verses[0];
        assert_eq!(verse.original.len(), 2);
        assert_eq!(verse.original[0].word, "בְּרֵאשִׁ֖ית");
        assert_eq!(verse.english.len(), 3);
        assert_eq!(verse.english[1].text, " ");
        assert!(verse.english[1].strongs.is_none());
    }

    #[test]
    fn a_partial_range_is_not_found_rather_than_returned_incomplete() {
        let conn = seeded_connection();
        // Only verse 1 is seeded; asking for 1-2 must not silently return
        // just verse 1's data.
        let result = get_interlinear(
            InterlinearQuery {
                reference: "Genesis 1:1-2".into(),
            },
            Some(&conn),
        );
        assert!(!result.found);
    }
}
