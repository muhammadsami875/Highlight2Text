"""Automated Workflow agent.

Trigger-based, not prompted: whenever the Organizer Agent (or a heuristic check here)
flags an uploaded file as an invoice, this agent extracts the vendor/amount/date/category
and logs a row to the expenses table with zero human intervention, then logs what it did
to the Activity Feed. Rows live in the database (not a CSV on disk) so the log survives
across serverless invocations; /api/expenses.csv renders it to CSV on demand.
"""
import datetime
import re

from .. import config, db, llm

INVOICE_HINTS = ("invoice", "total due", "amount due", "bill to", "remit payment")
CATEGORY_KEYWORDS = {
    "Infrastructure": ("hosting", "server", "cloud", "aws", "domain"),
    "Software": ("license", "subscription", "saas", "software"),
    "Travel": ("flight", "hotel", "travel", "airfare"),
    "Office": ("office", "supplies", "furniture"),
}


def looks_like_invoice(doc_type: str, text: str) -> bool:
    if doc_type == "invoice":
        return True
    lowered = text.lower()
    return any(h in lowered for h in INVOICE_HINTS)


def maybe_trigger(file_id: str, filename: str, text: str, doc_type: str) -> dict | None:
    if not looks_like_invoice(doc_type, text):
        return None

    if config.LIVE_MODE:
        row = _live_extract(filename, text)
    else:
        row = _mock_extract(filename, text)

    db.insert_expense(
        date=row["date"], vendor=row["vendor"], amount=row["amount"],
        category=row["category"], source_file=filename, file_id=file_id,
    )
    db.log_activity(
        "Workflow Agent",
        f"Detected an invoice in '{filename}' → logged ${row['amount']:.2f} to "
        f"{row['vendor']} under {row['category']} in the expense log.",
        file_id=file_id,
    )
    return row


def _live_extract(filename: str, text: str) -> dict:
    system = (
        "You are the Workflow Agent for an autonomous cloud drive. Extract invoice fields "
        "from the document. Respond with ONLY a JSON object, no prose."
    )
    user = (
        f"Filename: {filename}\nDocument:\n{text[:4000]}\n\n"
        'Return JSON: {"vendor": str, "amount": number (no currency symbol), '
        '"date": "YYYY-MM-DD", "category": one of '
        '["Infrastructure", "Software", "Travel", "Office", "General"]}'
    )
    try:
        data = llm.complete_json(system, user, max_tokens=300)
        return {
            "vendor": str(data.get("vendor", "Unknown vendor")),
            "amount": float(data.get("amount", 0) or 0),
            "date": str(data.get("date", datetime.date.today().isoformat())),
            "category": str(data.get("category", "General")),
        }
    except Exception:
        return _mock_extract(filename, text)


def _mock_extract(filename: str, text: str) -> dict:
    amounts = [
        float(a.replace(",", "")) for a in re.findall(r"\$\s?([\d,]+\.?\d*)", text)
    ]
    amount = max(amounts) if amounts else 0.0

    date_match = re.search(r"\d{4}-\d{2}-\d{2}", text)
    date = date_match.group(0) if date_match else datetime.date.today().isoformat()

    vendor_match = re.search(r"(?:from|vendor|bill to)[:\s]+([A-Za-z0-9 &.,'-]{3,60})", text, re.IGNORECASE)
    vendor = vendor_match.group(1).strip() if vendor_match else filename.rsplit(".", 1)[0].replace("_", " ").title()

    lowered = text.lower()
    category = "General"
    for cat, keywords in CATEGORY_KEYWORDS.items():
        if any(k in lowered for k in keywords):
            category = cat
            break

    return {"vendor": vendor, "amount": amount, "date": date, "category": category}


def read_expenses() -> list[dict]:
    return db.list_expenses()
