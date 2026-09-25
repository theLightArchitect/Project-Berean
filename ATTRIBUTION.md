# Attribution

Project Berean's corpus database (`engine/corpus.db`, built by
`engine/src/bin/ingest.rs`) is derived from
[BSB-publishing/bsb-data-output](https://github.com/BSB-publishing/bsb-data-output),
itself compiled from several open sources. Per that project's own
[ATTRIBUTION.md](https://github.com/BSB-publishing/bsb-data-output/blob/main/ATTRIBUTION.md),
here's what we actually use and what it requires:

## No attribution required (CC0 / public domain)

- **Berean Standard Bible** verse text — `base/text-only/` — public domain.

## Attribution required (CC BY 4.0)

We ingest two CC-BY-licensed directories, so this attribution is required
wherever the resulting data is used or displayed:

> Hebrew morphology data from [Open Scriptures Hebrew Bible (OSHB)](https://hb.openscriptures.org/),
> licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
>
> Lexicon data from the [STEPBible Data Repository](https://github.com/STEPBible/STEPBible-Data)
> (TBESH/TBESG extended Strong's lexicons), licensed under
> [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
> Credit: Tyndale House, Cambridge (www.STEPBible.org).

Specifically:
- `base/index-cc-by/` (cross-references and topics, keyed to OSHB-derived
  morphology) → powers `lookup_crossrefs`.
- `base/lexicon/` (`hebrew.json`, `greek.json`) → powers `lookup_lexicon`.

## Not yet used

`base/geography/`, `base/proper-names/`, and `base/versification/` are also
CC-BY / CC-BY-SA licensed in the upstream project but aren't ingested yet —
update this file when they are (see `docs/TOOL-PALETTE.md` for the planned
`lookup_place` tool, which will need the geography attribution too).
