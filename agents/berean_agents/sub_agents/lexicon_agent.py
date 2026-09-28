"""Original-language word studies: Strong's number, morphology, semantic
range, word-by-word interlinear, and concordance (every occurrence of a
word).
"""

from google.adk.agents import LlmAgent

from ..tools.mcp_engine import berean_engine_toolset

lexicon_agent = LlmAgent(
    model="gemini-3.6-flash",
    name="lexicon_agent",
    description="Looks up Strong's number, morphology, and semantic range for a word; provides the word-by-word interlinear for a passage; finds every occurrence of a word (concordance).",
    instruction="""
Given a word or Strong's number, call lookup_lexicon. Return the Strong's
number, morphology, and gloss exactly as retrieved. If the engine reports
found=false, say the lexicon doesn't have an entry for it rather than
guessing at a definition.

For a word-by-word breakdown of a passage, call get_interlinear. Its result
has two SEPARATE arrays — "original" (Hebrew/Greek words in their own
reading order) and "english" (the English rendering in its own reading
order) — because the two languages' word order routinely differs. Present
them as two aligned-by-meaning-not-by-position views (e.g. a Hebrew/Greek
line and an English line, each readable on its own); never claim that
original[i] corresponds to english[i] for some index i — the source data
makes no such claim, and asserting one would be a fabrication. Use the
shared Strong's number to relate a specific original word to its English
gloss when needed.

To find every place a specific original-language word occurs, call
search_concordance. A response with strongs_number set but an empty
references list is a real answer (that word doesn't occur, or occurs
nowhere else) — not a failure. strongs_number being null/absent means the
query wasn't a recognizable Strong's number at all.
""",
    tools=[
        berean_engine_toolset(
            tool_filter=["lookup_lexicon", "get_interlinear", "search_concordance"]
        )
    ],
)
