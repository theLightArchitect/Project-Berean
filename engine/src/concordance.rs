//! Concordance: every verse where a given Strong's number occurs, in
//! canonical Bible order. Same normalization discipline as lexicon.rs — a
//! query only succeeds if it's a genuine Strong's number.

use rusqlite::Connection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::reference::normalize_strongs;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConcordanceQuery {
    /// A Strong's number, e.g. "G26" or "H0430".
    pub strongs: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConcordanceResult {
    pub query: String,
    pub strongs_number: Option<String>,
    /// Every occurrence, in canonical Bible order. A genuine Strong's
    /// number with zero occurrences (a real, if unusual, answer) and "not a
    /// Strong's number at all" are both empty here — `strongs_number` is
    /// what tells them apart (Some vs. None).
    pub references: Vec<String>,
}

pub fn search_concordance(query: ConcordanceQuery, conn: Option<&Connection>) -> ConcordanceResult {
    let not_found = || ConcordanceResult {
        query: query.strongs.clone(),
        strongs_number: None,
        references: Vec::new(),
    };

    let Some(conn) = conn else {
        return not_found();
    };
    let Some(canonical) = normalize_strongs(&query.strongs) else {
        return not_found();
    };

    let mut stmt =
        match conn.prepare("SELECT target_ref FROM concordance WHERE strongs = ?1 ORDER BY seq") {
            Ok(stmt) => stmt,
            Err(_) => return not_found(),
        };
    let rows = stmt.query_map([&canonical], |row| row.get::<_, String>(0));
    let Ok(rows) = rows else {
        return not_found();
    };

    ConcordanceResult {
        query: query.strongs,
        strongs_number: Some(canonical),
        references: rows.flatten().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_fabricates_occurrences_for_an_unloaded_concordance() {
        let result = search_concordance(
            ConcordanceQuery {
                strongs: "G26".into(),
            },
            None,
        );
        assert!(result.strongs_number.is_none());
        assert!(result.references.is_empty());
    }

    #[test]
    fn rejects_a_non_strongs_query_rather_than_guess() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE concordance (strongs TEXT, seq INTEGER, target_ref TEXT);",
        )
        .unwrap();
        let result = search_concordance(
            ConcordanceQuery {
                strongs: "love".into(),
            },
            Some(&conn),
        );
        assert!(result.strongs_number.is_none());
    }

    #[test]
    fn finds_real_occurrences_in_canonical_order_and_normalizes_padding() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE concordance (strongs TEXT, seq INTEGER, target_ref TEXT);
             INSERT INTO concordance VALUES ('G26', 0, 'John 13:35');
             INSERT INTO concordance VALUES ('G26', 1, 'Romans 5:8');",
        )
        .unwrap();

        let result = search_concordance(
            ConcordanceQuery {
                strongs: "G0026".into(),
            },
            Some(&conn),
        );
        assert_eq!(result.strongs_number.as_deref(), Some("G26"));
        assert_eq!(result.references, vec!["John 13:35", "Romans 5:8"]);
    }

    #[test]
    fn a_valid_strongs_number_with_zero_occurrences_is_a_real_empty_answer() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE concordance (strongs TEXT, seq INTEGER, target_ref TEXT);",
        )
        .unwrap();
        let result = search_concordance(
            ConcordanceQuery {
                strongs: "G26".into(),
            },
            Some(&conn),
        );
        // Distinguishable from "not a Strong's number": strongs_number is
        // Some, just with no occurrences recorded.
        assert_eq!(result.strongs_number.as_deref(), Some("G26"));
        assert!(result.references.is_empty());
    }
}
