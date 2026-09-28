"""Shared connection to the Berean Engine (Rust MCP server).

The engine owns the cross-reference graph and lexicon data, enforcing the
same verbatim-citation contract as the grounding agent's Vertex AI Search
tool, just for structured lookups instead of full-text search. See
engine/src/corpus.rs for the contract: return the data with its source, or
say "not found" — never guess.
"""

import os

from google.adk.tools.mcp_tool import McpToolset
from google.adk.tools.mcp_tool.mcp_session_manager import StdioConnectionParams
from mcp import StdioServerParameters

_ENGINE_BIN = os.environ.get(
    "BEREAN_ENGINE_BIN",
    os.path.join(os.path.dirname(__file__), "..", "..", "..", "engine", "target", "release", "berean-engine"),
)

# The MCP Python SDK's stdio client does NOT inherit the parent process's
# full environment by default — it only passes a security allowlist (PATH,
# HOME, etc.), deliberately excluding app-specific vars. Without explicitly
# forwarding BEREAN_CORPUS_DB here, the spawned engine can never find the
# corpus database in any real deployment; it would silently fall back to
# the default relative "corpus.db" path, which only works by accident of
# working directory. Forward it explicitly whenever it's set.
_ENGINE_ENV = {}
if "BEREAN_CORPUS_DB" in os.environ:
    _ENGINE_ENV["BEREAN_CORPUS_DB"] = os.environ["BEREAN_CORPUS_DB"]


def berean_engine_toolset(tool_filter: list[str] | None = None) -> McpToolset:
    """Build an McpToolset connected to the Berean Engine over stdio.

    tool_filter restricts which engine tools a given agent can call (e.g. the
    lexicon agent should only ever see lookup_lexicon).
    """
    return McpToolset(
        connection_params=StdioConnectionParams(
            server_params=StdioServerParameters(
                command=_ENGINE_BIN, args=[], env=_ENGINE_ENV or None
            ),
            timeout=10,
        ),
        tool_filter=tool_filter,
    )
