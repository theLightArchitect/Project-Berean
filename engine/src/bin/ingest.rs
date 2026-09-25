//! Builds `corpus.db` from the open datasets described in
//! `docs/TOOL-PALETTE.md`: BSB verse text (public domain), BSB's CC-BY
//! cross-reference/topic index, and the BDB/Abbott-Smith-derived lexicon —
//! all sourced from BSB-publishing/bsb-data-output.
//!
//! Usage:
//!   cargo run --bin ingest -- <path-to-bsb-data-output> <output-db-path>
//!
//! `<path-to-bsb-data-output>` is a local clone of
//! https://github.com/BSB-publishing/bsb-data-output — not vendored into
//! this repo (it's tens of MB of upstream data); clone it yourself and pass
//! the path.
//!
//! Rebuilds from scratch every run (drops and recreates all tables) rather
//! than updating incrementally, so the output is always exactly what the
//! source data says — no accumulated drift from partial re-runs.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use berean_engine::reference::{book_codes, compact_id_to_display, normalize_strongs};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::Value;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: ingest <path-to-bsb-data-output> <output-db-path>");
        std::process::exit(2);
    }
    let data_dir = PathBuf::from(&args[1]);
    let db_path = &args[2];

    let mut conn = Connection::open(db_path)?;
    create_schema(&conn)?;

    let tx = conn.transaction()?;
    let passage_count = ingest_passages(&tx, &data_dir.join("base/text-only"))?;
    println!("ingested {passage_count} passages");

    let crossref_count = ingest_crossrefs(&tx, &data_dir.join("base/index-cc-by"))?;
    println!("ingested {crossref_count} cross-references");

    let lexicon_count = ingest_lexicon(&tx, &data_dir.join("base/lexicon"))?;
    println!("ingested {lexicon_count} lexicon entries");
    tx.commit()?;

    Ok(())
}

fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS passages;
        DROP TABLE IF EXISTS crossrefs;
        DROP TABLE IF EXISTS lexicon;

        CREATE TABLE passages (
            book TEXT NOT NULL,
            chapter INTEGER NOT NULL,
            verse INTEGER NOT NULL,
            translation TEXT NOT NULL,
            text TEXT NOT NULL,
            PRIMARY KEY (book, chapter, verse, translation)
        );

        CREATE TABLE crossrefs (
            book TEXT NOT NULL,
            chapter INTEGER NOT NULL,
            verse INTEGER NOT NULL,
            target_ref TEXT NOT NULL,
            source TEXT NOT NULL
        );
        CREATE INDEX idx_crossrefs_verse ON crossrefs(book, chapter, verse);

        CREATE TABLE lexicon (
            strongs TEXT PRIMARY KEY,
            language TEXT NOT NULL,
            lemma TEXT NOT NULL,
            transliteration TEXT,
            morphology TEXT,
            gloss TEXT,
            definition TEXT
        );
        ",
    )
}

/// `base/text-only/{BOOK}_{chapter:03}_BSB.txt`, one verse per line.
fn ingest_passages(conn: &Connection, dir: &Path) -> anyhow::Result<u64> {
    let mut count = 0u64;
    let mut stmt = conn.prepare(
        "INSERT INTO passages (book, chapter, verse, translation, text) VALUES (?1, ?2, ?3, 'BSB', ?4)",
    )?;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let Some(stem) = file_name.strip_suffix("_BSB.txt") else {
            continue;
        };
        let Some((book, chapter_str)) = stem.split_once('_') else {
            continue;
        };
        let Ok(chapter) = chapter_str.parse::<u32>() else {
            continue;
        };

        let content = fs::read_to_string(entry.path())?;
        for (i, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let verse = (i + 1) as u32;
            stmt.execute(rusqlite::params![book, chapter, verse, line])?;
            count += 1;
        }
    }
    Ok(count)
}

#[derive(Deserialize)]
struct IndexCcByEntry {
    #[serde(default)]
    x: Vec<String>,
}

/// `base/index-cc-by/{BOOK}/{BOOK}{chapter}.jsonl`, one verse per line, in
/// order starting at verse 1. Book codes are always exactly 3 characters in
/// this dataset (including the digit-prefixed ones like "1CH"), so the
/// first 3 characters of the filename stem are always the book code —
/// verified against the real book-code table rather than assumed.
fn ingest_crossrefs(conn: &Connection, dir: &Path) -> anyhow::Result<u64> {
    let known_codes: Vec<&str> = book_codes().collect();
    let mut count = 0u64;
    let mut stmt = conn.prepare(
        "INSERT INTO crossrefs (book, chapter, verse, target_ref, source) VALUES (?1, ?2, ?3, ?4, 'bsb-index-cc-by')",
    )?;

    for book_dir in fs::read_dir(dir)? {
        let book_dir = book_dir?;
        if !book_dir.file_type()?.is_dir() {
            continue;
        }
        for entry in fs::read_dir(book_dir.path())? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            let Some(stem) = file_name.strip_suffix(".jsonl") else {
                continue;
            };
            if stem.len() < 4 {
                continue;
            }
            let (book_candidate, chapter_str) = stem.split_at(3);
            if !known_codes.contains(&book_candidate) {
                continue;
            }
            let Ok(chapter) = chapter_str.parse::<u32>() else {
                continue;
            };

            let content = fs::read_to_string(entry.path())?;
            for (i, line) in content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let verse = (i + 1) as u32;
                let parsed: IndexCcByEntry = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                for target in parsed.x {
                    if let Some(display) = compact_id_to_display(&target) {
                        stmt.execute(rusqlite::params![book_candidate, chapter, verse, display])?;
                        count += 1;
                    }
                }
            }
        }
    }
    Ok(count)
}

#[derive(Deserialize)]
struct LexiconEntry {
    lemma: String,
    transliteration: Option<String>,
    morphology: Option<String>,
    gloss: Option<String>,
    definition: Option<String>,
}

/// `base/lexicon/{hebrew,greek}.json`: one big object keyed by Strong's
/// number, with duplicate zero-padded and unpadded keys for the same entry
/// (e.g. "H0001" and "H1"). Deduplicated via `normalize_strongs` so each
/// canonical key is inserted once.
fn ingest_lexicon(conn: &Connection, dir: &Path) -> anyhow::Result<u64> {
    let mut entries: HashMap<String, (&str, LexiconEntry)> = HashMap::new();

    for (file, language) in [("hebrew.json", "hebrew"), ("greek.json", "greek")] {
        let path = dir.join(file);
        let content = fs::read_to_string(&path)?;
        let raw: HashMap<String, Value> = serde_json::from_str(&content)?;
        for (key, value) in raw {
            let Some(canonical) = normalize_strongs(&key) else {
                continue;
            };
            if entries.contains_key(&canonical) {
                continue;
            }
            let Ok(entry) = serde_json::from_value::<LexiconEntry>(value) else {
                continue;
            };
            entries.insert(canonical, (language, entry));
        }
    }

    let mut stmt = conn.prepare(
        "INSERT INTO lexicon (strongs, language, lemma, transliteration, morphology, gloss, definition)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    let mut count = 0u64;
    for (strongs, (language, entry)) in &entries {
        stmt.execute(rusqlite::params![
            strongs,
            language,
            entry.lemma,
            entry.transliteration,
            entry.morphology,
            entry.gloss,
            entry.definition
        ])?;
        count += 1;
    }
    Ok(count)
}
