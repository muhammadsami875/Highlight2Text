"""Semantic Retrieval agent.

Takes a free-text question, pulls the most relevant chunks across every file in the
drive (PDFs, transcripts, spreadsheets alike) via search.search(), and synthesizes one
direct answer with inline [filename] citations instead of returning a list of links.
"""
from .. import config, llm, search


def ask(question: str, top_k: int = 6) -> dict:
    chunks = search.search(question, top_k=top_k)
    if not chunks:
        return {
            "answer": "Nothing in the drive looks related to that yet — try uploading "
                       "some files first.",
            "citations": [],
        }

    if config.LIVE_MODE:
        return _live_answer(question, chunks)
    return _mock_answer(question, chunks)


def _live_answer(question: str, chunks: list[dict]) -> dict:
    context = "\n\n".join(
        f"[{c['filename']}]\n{c['text']}" for c in chunks
    )
    system = (
        "You are the Retrieval Agent for an autonomous cloud drive. Answer the user's "
        "question using ONLY the provided excerpts. Every factual claim must carry an "
        "inline citation in the form [filename] right after it. If sources disagree, say so "
        "explicitly and cite both. If the excerpts don't contain the answer, say that plainly "
        "instead of guessing."
    )
    user = f"Question: {question}\n\nExcerpts:\n{context}"
    try:
        answer = llm.complete(system, user, max_tokens=700)
    except Exception as exc:  # pragma: no cover - defensive
        return _mock_answer(question, chunks, note=f"[live LLM call failed: {exc}]")
    citations = sorted({c["filename"] for c in chunks})
    return {"answer": answer, "citations": citations}


def _mock_answer(question: str, chunks: list[dict], note: str | None = None) -> dict:
    lines = [
        note or "[offline mode] Showing the best-matching passages verbatim — "
                "add ANTHROPIC_API_KEY for a synthesized, cited answer.",
        "",
    ]
    for c in chunks:
        snippet = c["text"].strip().replace("\n", " ")[:280]
        lines.append(f"• [{c['filename']}] {snippet}...")
    citations = sorted({c["filename"] for c in chunks})
    return {"answer": "\n".join(lines), "citations": citations}
