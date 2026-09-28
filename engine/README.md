# Berean Engine

Rust MCP server exposing twelve tools: `lookup_passage`, `lookup_crossrefs`,
`lookup_lexicon`, `get_interlinear`, `search_concordance`,
`lookup_manuscript_variants`, `lookup_confession`, `search_patristics`,
`compare_translations`, `detect_pastoral_signal`, `read_journal`,
`write_journal`. Runs over stdio so ADK's `McpToolset` can connect to it
directly (see `../agents/berean_agents/tools/mcp_engine.py`).

Every tool follows the same discipline, adapted per domain — see
`docs/ARCHITECTURE.md` for the full table:

- Content lookups (`corpus`, `crossref`, `lexicon`, `interlinear`,
  `concordance`, `criticism`, `confessions`, `patristics`, `translations`)
  return the data with its exact source, or say "not found" — never
  fabricate.
- `interlinear.rs` specifically: original-language and English word arrays
  are each in their own reading order and are never positionally zipped —
  Hebrew/Greek word order routinely differs from English syntax, so
  claiming word[i] corresponds to word[i] across the two arrays would
  misrepresent the source data.
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

`lookup_passage`, `lookup_crossrefs`, `lookup_lexicon`, `get_interlinear`,
and `search_concordance` are backed by a local SQLite database
(`corpus.db`) built from open datasets — see `../docs/TOOL-PALETTE.md` and
`../ATTRIBUTION.md` for what's in it and the attribution it requires.
Without this database built, those tools correctly return `found: false` /
empty results for everything (the never-fabricate contract holds either
way — an empty corpus is just an honest one).

```bash
git clone --depth 1 https://github.com/BSB-publishing/bsb-data-output /tmp/bsb-data-output
cargo run --release --bin ingest -- /tmp/bsb-data-output ./corpus.db
```

Point the server at it (defaults to `./corpus.db` in the working directory
if unset):

```bash
export BEREAN_CORPUS_DB=./corpus.db
```

**If you're launching the engine from a Python MCP client** (ADK's
`McpToolset`, or any client built on the official MCP Python SDK): the
spawned subprocess does **not** inherit your shell's environment by
default — the SDK only forwards a security allowlist (`PATH`, `HOME`,
etc.). You must explicitly pass `BEREAN_CORPUS_DB` via the client's
`env=` parameter, or the engine silently falls back to the default
relative path. `../agents/berean_agents/tools/mcp_engine.py` does this
correctly — see its `_ENGINE_ENV` handling if you're writing a different
client.

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

`lookup_passage`, `lookup_crossrefs`, `lookup_lexicon`, `get_interlinear`,
and `search_concordance` are backed by real data once `corpus.db` is built
(BSB verse text, ~430k TSK-derived cross-references, ~19k Strong's lexicon
entries, ~437k original-language words + ~822k English segments, ~372k
concordance entries — see the ingest counts printed by the `ingest` binary).
`lookup_manuscript_variants`, `lookup_confession`, `search_patristics`, and
`compare_translations` are still stubs — their data sources are the next
step (see `docs/TOOL-PALETTE.md`).
