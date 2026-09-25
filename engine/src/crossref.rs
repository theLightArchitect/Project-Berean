//! Cross-reference constellation lookups. Curated (human-verified) edges and
//! AI-suggested edges are always returned as separate lists — never merged —
//! so a client can't accidentally present a model guess as an established
//! typological link.

use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::reference;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CrossRefQuery {
    pub reference: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossRefResult {
    pub reference: String,
    pub curated: Vec<String>,
    pub ai_suggested: Vec<String>,
}

/// `curated` is backed by the corpus database (BSB's cross-reference index,
/// itself compiled from the Treasury of Scripture Knowledge — see
/// `docs/TOOL-PALETTE.md`). `ai_suggested` (embedding-similarity search over
/// the corpus) isn't implemented yet — it stays empty rather than
/// fabricated, same as `curated` does when the database isn't loaded.
pub fn lookup_crossrefs(query: CrossRefQuery, conn: Option<&Connection>) -> CrossRefResult {
    let empty = || CrossRefResult {
        reference: query.reference.clone(),
        curated: Vec::new(),
        ai_suggested: Vec::new(),
    };

    let Some(conn) = conn else {
        return empty();
    };
    let Some(parsed) = reference::parse(&query.reference) else {
        return empty();
    };

    let mut stmt = match conn.prepare(
        "SELECT DISTINCT target_ref FROM crossrefs
         WHERE book = ?1 AND chapter = ?2 AND verse BETWEEN ?3 AND ?4
         ORDER BY target_ref",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return empty(),
    };

    let rows = stmt.query_map(
        rusqlite::params![
            parsed.book,
            parsed.chapter,
            parsed.verse_start,
            parsed.verse_end
        ],
        |row| row.get::<_, String>(0),
    );

    let Ok(rows) = rows else {
        return empty();
    };

    CrossRefResult {
        reference: query.reference,
        curated: rows.flatten().collect(),
        ai_suggested: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_and_ai_suggested_never_share_entries_when_unloaded() {
        let result = lookup_crossrefs(
            CrossRefQuery {
                reference: "Exodus 12:1-13".into(),
            },
            None,
        );
        assert!(result.curated.is_empty());
        assert!(result.ai_suggested.is_empty());
    }

    #[test]
    fn finds_real_curated_crossrefs_when_the_corpus_is_loaded() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE crossrefs (book TEXT, chapter INTEGER, verse INTEGER, target_ref TEXT, source TEXT);
             INSERT INTO crossrefs VALUES ('JHN', 1, 1, 'Genesis 1:1', 'bsb-index-cc-by');
             INSERT INTO crossrefs VALUES ('JHN', 1, 1, 'Colossians 1:17', 'bsb-index-cc-by');",
        )
        .unwrap();

        let result = lookup_crossrefs(
            CrossRefQuery {
                reference: "John 1:1".into(),
            },
            Some(&conn),
        );
        assert_eq!(result.curated, vec!["Colossians 1:17", "Genesis 1:1"]);
        assert!(result.ai_suggested.is_empty());
    }
}
