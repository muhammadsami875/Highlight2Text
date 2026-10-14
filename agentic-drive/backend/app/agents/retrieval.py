"""Semantic Retrieval agent.

Takes a free-text question, pulls the most relevant chunks across every file in the
drive (PDFs, transcripts, spreadsheets alike) via search.search()'s TF-IDF ranking, and
returns the best-matching passages with inline [filename] citations — no external API
call, just ranked retrieval over the drive's own content.
"""
from .. import search


def ask(question: str, top_k: int = 6) -> dict:
    chunks = search.search(question, top_k=top_k)
    if not chunks:
        return {
            "answer": "Nothing in the drive looks related to that yet — try uploading "
                       "some files first.",
            "citations": [],
        }

    lines = []
    for c in chunks:
        snippet = c["text"].strip().replace("\n", " ")[:280]
        lines.append(f"• [{c['filename']}] {snippet}...")
    citations = sorted({c["filename"] for c in chunks})
    return {"answer": "\n".join(lines), "citations": citations}
