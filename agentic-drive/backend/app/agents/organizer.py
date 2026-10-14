"""Proactive Organization agent.

Runs on every file the moment it lands in the drive. No folders are ever created
by the user — this agent decides tags, a topic cluster (reusing an existing one when
it fits), a one-line summary, and a document type, purely from the file's own content
(keyword overlap, lightweight pattern matching) with no external API call — and it
always records *why* it filed the file where it did so the decision stays legible to
the user.
"""
import re
from collections import Counter

from .. import db

STOPWORDS = {
    "the", "and", "for", "that", "this", "with", "from", "have", "will", "about",
    "into", "your", "their", "they", "what", "when", "where", "which", "there",
    "been", "were", "are", "was", "but", "not", "you", "our", "its", "can", "all",
    "has", "had", "per", "per.", "per,", "also", "any", "each",
}

INVOICE_HINTS = ("invoice", "total due", "amount due", "bill to", "remit payment")
MEETING_HINTS = ("transcript", "attendees:", "agenda", "meeting notes")


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


def organize(filename: str, text: str) -> dict:
    existing_clusters = db.existing_clusters()
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
                f"Grouped with '{existing}' because this document shares keywords "
                f"with files already filed there."
            )
            break
    if not cluster:
        headline = keywords[0].title() if keywords else "General"
        cluster = f"{headline} Documents"
        reason = (
            f"No existing group shared enough keywords, so a new cluster "
            f"'{cluster}' was started."
        )

    first_sentence = re.split(r"(?<=[.!?])\s", text.strip())[0][:200] if text.strip() else filename
    return {
        "doc_type": doc_type,
        "tags": tags,
        "cluster": cluster,
        "cluster_reason": reason,
        "summary": first_sentence,
    }
