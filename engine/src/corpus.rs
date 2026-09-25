//! Verbatim scripture retrieval. The one rule that matters: if a passage
//! isn't in the corpus, say so — never fabricate text to fill the gap.

use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::reference;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PassageQuery {
    /// e.g. "John 1:1-5"
    pub reference: String,
    /// e.g. "ESV", "NASB", "LXX"
    pub translation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassageResult {
    pub reference: String,
    pub translation: String,
    pub text: Option<String>,
    pub found: bool,
}

/// Looks up a passage against the corpus database (see `db.rs` and
/// `src/bin/ingest.rs`). `conn: None` (no database built or configured yet)
/// and "reference didn't parse" and "some verse in the range is missing"
/// all collapse to the same `found: false` — a partial range is never
/// silently returned as if it were complete.
pub fn lookup_passage(query: PassageQuery, conn: Option<&Connection>) -> PassageResult {
    let not_found = || PassageResult {
        reference: query.reference.clone(),
        translation: query.translation.clone(),
        text: None,
        found: false,
    };

    let Some(conn) = conn else {
        return not_found();
    };
    let Some(parsed) = reference::parse(&query.reference) else {
        return not_found();
    };

    let mut stmt = match conn.prepare(
        "SELECT verse, text FROM passages
         WHERE book = ?1 AND chapter = ?2 AND translation = ?3 AND verse BETWEEN ?4 AND ?5
         ORDER BY verse",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return not_found(),
    };

    let rows = stmt.query_map(
        rusqlite::params![
            parsed.book,
            parsed.chapter,
            query.translation,
            parsed.verse_start,
            parsed.verse_end
        ],
        |row| Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?)),
    );

    let Ok(rows) = rows else {
        return not_found();
    };

    let mut verses: Vec<(u32, String)> = Vec::new();
    for row in rows.flatten() {
        verses.push(row);
    }

    let expected = (parsed.verse_end - parsed.verse_start + 1) as usize;
    if verses.len() != expected {
        return not_found();
    }

    let text = verses
        .into_iter()
        .map(|(_, t)| t)
        .collect::<Vec<_>>()
        .join(" ");

    PassageResult {
        reference: query.reference,
        translation: query.translation,
        text: Some(text),
        found: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_fabricates_text_for_an_unloaded_corpus() {
        let result = lookup_passage(
            PassageQuery {
                reference: "John 1:1".into(),
                translation: "ESV".into(),
            },
            None,
        );
        assert!(!result.found);
        assert!(result.text.is_none());
    }

    fn seeded_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE passages (book TEXT, chapter INTEGER, verse INTEGER, translation TEXT, text TEXT);
             INSERT INTO passages VALUES ('JHN', 3, 16, 'BSB', 'For God so loved the world...');
             INSERT INTO passages VALUES ('GEN', 1, 1, 'BSB', 'In the beginning God created the heavens and the earth.');
             INSERT INTO passages VALUES ('GEN', 1, 2, 'BSB', 'Now the earth was formless and void...');",
        )
        .unwrap();
        conn
    }

    #[test]
    fn finds_a_real_single_verse_when_the_corpus_is_loaded() {
        let conn = seeded_connection();
        let result = lookup_passage(
            PassageQuery {
                reference: "John 3:16".into(),
                translation: "BSB".into(),
            },
            Some(&conn),
        );
        assert!(result.found);
        assert_eq!(
            result.text.as_deref(),
            Some("For God so loved the world...")
        );
    }

    #[test]
    fn concatenates_a_verse_range_in_order() {
        let conn = seeded_connection();
        let result = lookup_passage(
            PassageQuery {
                reference: "Genesis 1:1-2".into(),
                translation: "BSB".into(),
            },
            Some(&conn),
        );
        assert!(result.found);
        assert_eq!(
            result.text.as_deref(),
            Some("In the beginning God created the heavens and the earth. Now the earth was formless and void...")
        );
    }

    #[test]
    fn a_partial_range_is_reported_not_found_rather_than_returned_incomplete() {
        let conn = seeded_connection();
        // Genesis 1:1-5 is only partially in this seeded fixture (verses 1-2 only).
        let result = lookup_passage(
            PassageQuery {
                reference: "Genesis 1:1-5".into(),
                translation: "BSB".into(),
            },
            Some(&conn),
        );
        assert!(!result.found);
        assert!(result.text.is_none());
    }

    #[test]
    fn an_unparseable_reference_is_not_found_even_with_a_loaded_corpus() {
        let conn = seeded_connection();
        let result = lookup_passage(
            PassageQuery {
                reference: "not a reference".into(),
                translation: "BSB".into(),
            },
            Some(&conn),
        );
        assert!(!result.found);
    }
}
