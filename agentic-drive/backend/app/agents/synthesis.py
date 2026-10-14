"""Cross-Document Synthesis agent.

Reads several existing files at once and assembles a new document answering the
user's instruction (e.g. "draft a unified PRD from these three transcripts") by
merging their summaries and leading content — no external API call, a mechanical
but legible merge. The result is not a disposable chat reply — it is written back
into the drive as a first-class file, auto-tagged, clustered, and stamped with
provenance (`source_file_ids`) so it's clear which files it was generated from.
"""
import re
from collections import Counter

from .. import db


def synthesize(file_ids: list[str], instruction: str) -> dict:
    sources = db.get_files(file_ids)
    if not sources:
        raise ValueError("None of the selected files exist")

    title = instruction.strip().rstrip(".") or "Synthesized Document"
    parts = []
    for s in sources:
        excerpt = s["text"].strip().replace("\n", " ")[:400]
        parts.append(f"## From {s['filename']}")
        parts.append(f"{s['summary']}\n\n{excerpt}...")
        parts.append("")
    body = "\n".join(parts)

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
        cluster_reason="Filed alongside its sources' cluster because it was generated from them.",
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


def _slug(title: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
    return slug or db.new_id()
