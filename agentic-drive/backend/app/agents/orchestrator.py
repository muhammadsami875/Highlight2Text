"""Thin router in front of the Retrieval and Synthesis agents.

The chat box is a single text input; this is the "Routing & Synthesis" layer from the
brief's diagram. If the user selected one or more files AND the message reads like a
drafting instruction, it's routed to the Synthesis agent; otherwise it's a question and
goes to the Retrieval agent.
"""
from . import retrieval, synthesis

DRAFT_VERBS = ("draft", "write", "generate", "create", "compose", "produce", "merge", "combine", "summarize")


def handle(message: str, selected_file_ids: list[str] | None = None) -> dict:
    selected_file_ids = selected_file_ids or []
    looks_like_drafting = any(v in message.lower() for v in DRAFT_VERBS)

    if selected_file_ids and looks_like_drafting:
        result = synthesis.synthesize(selected_file_ids, message)
        return {
            "agent": "synthesis",
            "answer": f"Drafted **{result['title']}** and saved it to the drive as "
                      f"`{result['filename']}`.",
            "citations": [],
            "generated_file_id": result["file_id"],
        }

    result = retrieval.ask(message)
    return {
        "agent": "retrieval",
        "answer": result["answer"],
        "citations": result["citations"],
        "generated_file_id": None,
    }
