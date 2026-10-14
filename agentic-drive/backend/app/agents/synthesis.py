"""Cross-Document Synthesis agent.

Reads several existing files at once and drafts a new document answering the user's
instruction (e.g. "draft a unified PRD from these three transcripts"). The result is
not a disposable chat reply — it is written back into the drive as a first-class file,
auto-tagged, clustered, and stamped with provenance (`source_file_ids`) so it's clear
which files it was generated from.
"""
import re
from collections import Counter

from .. import config, db, llm


def synthesize(file_ids: list[str], instruction: str) -> dict:
    sources = db.get_files(file_ids)
    if not sources:
        raise ValueError("None of the selected files exist")

    if config.LIVE_MODE:
        title, body = _live_draft(sources, instruction)
    else:
        title, body = _mock_draft(sources, instruction)

    filename = _slug(title) + ".md"
    content = f"# {title}\n\n{body}\n\n---\n*Generated from: {', '.join(s['filename'] for s in sources)}*\n"

    tags = sorted({t for s in sources for t in s["tags"]} | {"synthesis"})
    cluster_counts = Counter(s["cluster"] for s in sources)
    cluster = cluster_counts.most_common(1)[0][0] if cluster_counts else "Synthesis Drafts"

    file_id = db.insert_file(
        filename=filename,
        text=content,
        summary=f"Synthesized from {len(sources)} file(s): {instruction[:120]}",
        tags=list(tags),
        cluster=cluster,
        cluster_reason=f"Filed alongside its sources' cluster because it was generated from them.",
        doc_type="synthesis",
        source_file_ids=file_ids,
    )
    db.log_activity(
        "Synthesis Agent",
        f"Drafted '{title}' from {len(sources)} file(s) ({', '.join(s['filename'] for s in sources)}) "
        f"and filed it under '{cluster}'.",
        file_id=file_id,
    )
    return {"file_id": file_id, "filename": filename, "title": title, "content": content}


def _live_draft(sources: list[dict], instruction: str):
    joined = "\n\n".join(
        f"=== {s['filename']} ===\n{s['text'][:5000]}" for s in sources
    )
    system = (
        "You are the Synthesis Agent for an autonomous cloud drive. You read multiple source "
        "documents and produce ONE new, well-structured markdown document that fulfills the "
        "user's instruction. Resolve overlaps and call out any contradictions between sources "
        "explicitly rather than silently picking one. Do not include a top-level title heading "
        "in the body — that is added separately."
    )
    user = f"Instruction: {instruction}\n\nSource documents:\n{joined}"
    body = llm.complete(system, user, max_tokens=1500)
    title_system = "Return ONLY a short (<=8 word) title for the document described, no quotes, no punctuation at the end."
    try:
        title = llm.complete(title_system, instruction, max_tokens=30).strip().strip('"')
    except Exception:
        title = instruction[:60]
    return title or instruction[:60], body


def _mock_draft(sources: list[dict], instruction: str):
    title = instruction.strip().rstrip(".") or "Synthesized Document"
    parts = [
        "_[offline mode] This is a mechanical merge of the source documents — "
        "add ANTHROPIC_API_KEY for a real written synthesis._",
        "",
    ]
    for s in sources:
        excerpt = s["text"].strip().replace("\n", " ")[:400]
        parts.append(f"## From {s['filename']}")
        parts.append(f"{s['summary']}\n\n{excerpt}...")
        parts.append("")
    return title[:80], "\n".join(parts)


def _slug(title: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
    return slug or db.new_id()
