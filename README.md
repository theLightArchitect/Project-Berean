# Project-Berean

Next-gen Bible study platform that leverages compound A.I. infrastructure
systems for a whole new immersive experience for studying the Holy Bible.
All faiths and beliefs are welcome to try and get their questions answered
grounded by scripture.

Named for Acts 17:11: *"these were more noble... they searched the
scriptures daily to see if these things were so."* That's the design
principle, not just the name: the app shows its receipts. Every answer is
traceable back to the actual text, and the app never collapses a genuinely
contested question into one confident-sounding position.

## Structure

```
docs/          — architecture and design writeups (start with ARCHITECTURE.md)
agents/        — ADK (Agent Development Kit) multi-agent system, deployed to
                  the Gemini Enterprise Agent Platform
engine/        — Berean Engine: Rust MCP server owning the verbatim-citation
                  contract, cross-reference graph, lexicon, and pastoral/
                  journal safety contracts
infra/         — GCP Terraform: API enablement, AlloyDB, Vertex AI Search
                  datastores, Cloud Run placeholders
```

## Status

Agent topology and tool contracts are defined. Scripture text, curated
cross-references, and the Strong's lexicon are real, open-licensed data
(31k+ verses, ~430k cross-references, ~19k lexicon entries — see
`engine/README.md` to build the database). Manuscript variants,
confessional documents, patristic citations, and translation comparison are
still stubs — every lookup honestly returns "not found" rather than a
placeholder answer (see `engine/src/corpus.rs` and its sibling modules for
the enforced contract).

See `docs/ARCHITECTURE.md` for the full design, `docs/TOOL-PALETTE.md` for
the research-backed data/tool roadmap, `ATTRIBUTION.md` for the open-data
credit this project requires, and the READMEs in `agents/` and `engine/` for
how to build and run each piece.

## License

AGPL-3.0 — see `LICENSE`.
