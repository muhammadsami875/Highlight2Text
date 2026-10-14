"""Thin wrapper around the Claude API.

Every agent that calls an LLM goes through `complete()` / `complete_json()` here.
When no ANTHROPIC_API_KEY is configured, `config.LIVE_MODE` is False and callers
are expected to fall back to their own deterministic heuristic — see each agent's
`_mock_*` function. This keeps the whole app runnable and demoable with zero setup,
and gives real agentic behavior the moment a key is added.
"""
import json
import re

from . import config

_client = None


def _get_client():
    global _client
    if _client is None:
        import anthropic

        _client = anthropic.Anthropic(api_key=config.ANTHROPIC_API_KEY)
    return _client


def complete(system: str, user: str, max_tokens: int = 1024) -> str:
    if not config.LIVE_MODE:
        raise RuntimeError("LLM called while not in live mode — check config.LIVE_MODE first")
    client = _get_client()
    resp = client.messages.create(
        model=config.ANTHROPIC_MODEL,
        max_tokens=max_tokens,
        system=system,
        messages=[{"role": "user", "content": user}],
    )
    return "".join(block.text for block in resp.content if block.type == "text")


def complete_json(system: str, user: str, max_tokens: int = 1024) -> dict:
    """Ask the model for strict JSON and parse it, tolerating stray markdown fences."""
    raw = complete(system, user, max_tokens=max_tokens)
    match = re.search(r"\{.*\}", raw, re.DOTALL)
    candidate = match.group(0) if match else raw
    return json.loads(candidate)
