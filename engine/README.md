# Berean Engine

Rust MCP server exposing ten tools: `lookup_passage`, `lookup_crossrefs`,
`lookup_lexicon`, `lookup_manuscript_variants`, `lookup_confession`,
`search_patristics`, `compare_translations`, `detect_pastoral_signal`,
`read_journal`, `write_journal`. Runs over stdio so ADK's `McpToolset` can
connect to it directly (see `../agents/berean_agents/tools/mcp_engine.py`).

Every tool follows the same discipline, adapted per domain — see
`docs/ARCHITECTURE.md` for the full table:

- Content lookups (`corpus`, `crossref`, `lexicon`, `criticism`,
  `confessions`, `patristics`, `translations`) return the data with its exact
  source, or say "not found" — never fabricate.
- `pastoral.rs` never defaults to "confirmed safe" — `classified: false`
  means "not yet classified," not "no concern."
- `journal.rs` never claims a save that didn't happen — `persisted: false`
  until real durable storage exists.

## Build & test

```bash
cargo build --release
cargo test
```

Builds two binaries: `berean-engine` (the MCP server) and `ingest` (the
corpus database builder, below).

## Building the corpus database

`lookup_passage`, `lookup_crossrefs`, and `lookup_lexicon` are backed by a
local SQLite database (`corpus.db`) built from open datasets — see
`../docs/TOOL-PALETTE.md` and `../ATTRIBUTION.md` for what's in it and the
attribution it requires. Without this database built, those three tools
correctly return `found: false` for everything (the never-fabricate
contract holds either way — an empty corpus is just an honest one).

```bash
git clone --depth 1 https://github.com/BSB-publishing/bsb-data-output /tmp/bsb-data-output
cargo run --release --bin ingest -- /tmp/bsb-data-output ./corpus.db
```

Point the server at it (defaults to `./corpus.db` in the working directory
if unset):

```bash
export BEREAN_CORPUS_DB=./corpus.db
```

`lookup_manuscript_variants`, `lookup_confession`, `search_patristics`, and
`compare_translations` don't have a data source wired in yet — they still
honestly report not-found/unchecked for everything, per their modules'
contracts.

## Smoke-test the MCP handshake directly

```bash
{
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke-test","version":"0.1"}}}'
  echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"lookup_passage","arguments":{"reference":"John 3:16","translation":"BSB"}}}'
} | ./target/release/berean-engine
```

With `corpus.db` built, this returns the actual BSB text of John 3:16
instead of `found: false`.

## Status

`lookup_passage`, `lookup_crossrefs`, and `lookup_lexicon` are backed by real
data once `corpus.db` is built (BSB verse text, ~430k TSK-derived
cross-references, ~19k Strong's lexicon entries — see the ingest counts
above). `lookup_manuscript_variants`, `lookup_confession`,
`search_patristics`, and `compare_translations` are still stubs — their
data sources are the next step (see `docs/TOOL-PALETTE.md`).
