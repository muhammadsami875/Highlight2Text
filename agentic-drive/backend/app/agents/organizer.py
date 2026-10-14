"""Proactive Organization agent.

Runs on every file the moment it lands in the drive. No folders are ever created
by the user — this agent decides tags, a topic cluster (reusing an existing one when
it fits), a one-line summary, and a document type, and it always records *why* it
filed the file where it did so the decision stays legible to the user.
"""
import re
from collections import Counter

from .. import config, db, llm

STOPWORDS = {
    "the", "and", "for", "that", "this", "with", "from", "have", "will", "about",
    "into", "your", "their", "they", "what", "when", "where", "which", "there",
    "been", "were", "are", "was", "but", "not", "you", "our", "its", "can", "all",
    "has", "had", "per", "per.", "per,", "also", "any", "each",
}

INVOICE_HINTS = ("invoice", "total due", "amount due", "bill to", "remit payment")
MEETING_HINTS = ("transcript", "attendees:", "agenda", "meeting notes")


def organize(filename: str, text: str) -> dict:
    existing = db.existing_clusters()
    if config.LIVE_MODE:
        return _live_organize(filename, text, existing)
    return _mock_organize(filename, text, existing)


def _live_organize(filename: str, text: str, existing_clusters: list[str]) -> dict:
    excerpt = text[:6000]
    clusters_hint = ", ".join(existing_clusters) if existing_clusters else "(none yet)"
    system = (
        "You are the Organizer Agent inside an autonomous cloud drive. You file every "
        "incoming document with no human folder structure to rely on. Respond with ONLY "
        "a JSON object, no prose, no markdown fences."
    )
    user = (
        f"Filename: {filename}\n"
        f"Existing project clusters in the drive: {clusters_hint}\n\n"
        f"Document content (may be truncated):\n{excerpt}\n\n"
        "Return JSON with exactly these keys:\n"
        '  "doc_type": one of ["meeting_transcript", "invoice", "spreadsheet", "report", "other"],\n'
        '  "tags": 3-6 short topical tags (list of strings),\n'
        '  "cluster": the best existing cluster name if this document clearly belongs with it, '
        'otherwise a short new cluster/project name you propose (string),\n'
        '  "cluster_reason": one sentence explaining the filing decision in plain language, '
        "written as if explaining to the user what you did and why (string),\n"
        '  "summary": one sentence summarizing the document (string).'
    )
    try:
        data = llm.complete_json(system, user)
        return {
            "doc_type": data.get("doc_type", "other"),
            "tags": [str(t) for t in data.get("tags", [])][:6],
            "cluster": str(data.get("cluster", "Unsorted")).strip() or "Unsorted",
            "cluster_reason": str(data.get("cluster_reason", "")),
            "summary": str(data.get("summary", "")),
        }
    except Exception:
        return _mock_organize(filename, text, existing_clusters)


def _guess_doc_type(text_lower: str, filename: str) -> str:
    if filename.lower().endswith((".csv", ".xlsx", ".xls")):
        return "spreadsheet"
    if any(h in text_lower for h in INVOICE_HINTS):
        return "invoice"
    if any(h in text_lower for h in MEETING_HINTS) or re.search(r"\n[A-Z][a-z]+:\s", text_lower):
        return "meeting_transcript"
    return "other"


def _top_keywords(text: str, k: int = 5) -> list[str]:
    words = re.findall(r"[A-Za-z]{4,}", text.lower())
    words = [w for w in words if w not in STOPWORDS]
    counts = Counter(words)
    return [w for w, _ in counts.most_common(k)]


def _mock_organize(filename: str, text: str, existing_clusters: list[str]) -> dict:
    text_lower = text.lower()
    doc_type = _guess_doc_type(text_lower, filename)
    keywords = _top_keywords(text, 6)
    tags = keywords[:5] or [doc_type]

    cluster, reason = None, None
    for existing in existing_clusters:
        existing_words = set(re.findall(r"[a-z]{4,}", existing.lower()))
        if existing_words & set(keywords):
            cluster = existing
            reason = (
                f"[offline heuristic] Grouped with '{existing}' because this document shares "
                f"keywords with files already filed there."
            )
            break
    if not cluster:
        headline = keywords[0].title() if keywords else "General"
        cluster = f"{headline} Documents"
        reason = (
            f"[offline heuristic] No existing group shared enough keywords, so a new cluster "
            f"'{cluster}' was started. Add ANTHROPIC_API_KEY for topic-aware clustering."
        )

    first_sentence = re.split(r"(?<=[.!?])\s", text.strip())[0][:200] if text.strip() else filename
    return {
        "doc_type": doc_type,
        "tags": tags,
        "cluster": cluster,
        "cluster_reason": reason,
        "summary": first_sentence,
    }
