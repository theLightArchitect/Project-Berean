//! Corpus database connection. Local/dev backing store: a SQLite file built
//! by `src/bin/ingest.rs` from the open datasets named in
//! `docs/TOOL-PALETTE.md` (BSB text, cross-references, lexicon). The schema
//! here is deliberately simple enough to port to AlloyDB later without
//! changing any tool's contract — only this module and the ingest binary
//! would need to change.
//!
//! No database file present is a normal, expected state (a fresh checkout
//! before running the ingest) — every lookup module treats "no connection"
//! exactly like "not found in the connection", never as an error to surface
//! differently. That's what keeps the never-fabricate contract honest
//! whether or not the corpus has been built yet.

use rusqlite::{Connection, OpenFlags};
use std::env;

const DEFAULT_DB_PATH: &str = "corpus.db";

fn db_path() -> String {
    env::var("BEREAN_CORPUS_DB").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string())
}

/// Open the corpus database read-only. Returns `None` if it doesn't exist
/// yet or can't be opened — callers treat that exactly like "not found",
/// never as a reason to guess.
pub fn open() -> Option<Connection> {
    Connection::open_with_flags(db_path(), OpenFlags::SQLITE_OPEN_READ_ONLY).ok()
}
