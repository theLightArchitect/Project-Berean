//! Library crate shared by the MCP server binary (`src/main.rs`) and the
//! corpus ingestion binary (`src/bin/ingest.rs`), so both work against the
//! exact same reference parser, database schema helpers, and domain
//! contracts — no duplicated book-code tables or query logic to drift apart.

pub mod confessions;
pub mod corpus;
pub mod criticism;
pub mod crossref;
pub mod db;
pub mod engine;
pub mod journal;
pub mod lexicon;
pub mod pastoral;
pub mod patristics;
pub mod reference;
pub mod translations;
