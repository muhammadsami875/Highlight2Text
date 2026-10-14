# Agentic Drive

A prototype "Advanced Google Drive" — an active workspace where autonomous agents
organize, explain, synthesize, and act on your files instead of just storing them.

See **[BRAINSTORM.md](./BRAINSTORM.md)** for the conceptual vision and the agentic
user-flows this was designed from before any code was written.

## What it does

Every file that enters the drive goes through one ingestion pipeline, and four small
agents sit on top of it:

| Agent | Brief's capability | What it does |
|---|---|---|
| **Organizer** | Proactive Organization | Tags every file, assigns it to an existing cluster or starts a new one, and writes a one-line explanation of *why* — no manual folders, ever. |
| **Retrieval** | Semantic Retrieval | Answers free-text questions ("What was the budget discussed in the Q3 marketing meetings?") by reading across every file and citing sources inline. |
| **Synthesis** | Cross-Document Synthesis | Reads several selected files and drafts a new document from an instruction (e.g. "draft a unified PRD from these"), then **saves the result back into the drive** as a first-class, tagged, provenance-stamped file. |
| **Workflow** | Automated Workflows | Triggers automatically (no prompt) when an uploaded file looks like an invoice: extracts vendor/amount/date/category and appends a row to `expenses.csv`. |

A thin **Orchestrator** routes the chat box's free text to Retrieval or Synthesis
depending on whether you've selected files and whether the message reads like a
drafting instruction — this is the "Routing & Synthesis" middleware layer from the
brief's diagram.

Every agent action — a filing decision, a synthesis, an expense logged — is written to
an **Activity Feed** so autonomous behavior stays legible instead of opaque.

## Architecture

```
┌─────────────────────────────── Frontend (static HTML/JS) ───────────────────────────────┐
│  Workspaces (clusters/files)   │   Ask your drive (chat)   │   Agent activity + expenses │
└───────────────┬─────────────────────────────┬──────────────────────────┬─────────────────┘
                 │ upload                      │ ask / synthesize         │ poll
                 ▼                             ▼                          ▼
┌───────────────────────────────── FastAPI (backend/app) ──────────────────────────────────┐
│  Ingestion pipeline:  save → extract text → Organizer Agent → index → Workflow Agent      │
│  Chat pipeline:       Orchestrator → Retrieval Agent  |  Synthesis Agent (writes back)     │
│  Storage:             SQLite (files, tags, clusters, activity log) + flat files on disk    │
│  Retrieval:           TF-IDF over paragraph chunks (sklearn) → LLM reads top chunks        │
│  LLM:                 Claude (Anthropic API) when ANTHROPIC_API_KEY is set, else a         │
│                       deterministic heuristic fallback per agent ("mock mode")             │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

**Why TF-IDF + LLM instead of a hosted vector DB:** for a prototype, a real embedding
model dependency (or a paid embeddings API) is the single most likely thing to make the
demo fail in front of judges on an unfamiliar machine. TF-IDF retrieval + an LLM that
reads the retrieved passages and synthesizes a cited answer gets the same user-facing
behavior ("ask anything, get one cited answer") with zero external dependencies beyond
the LLM call itself, and it degrades gracefully (see "Mock mode" below) even without a
key. Swapping the retriever for real embeddings (Voyage, OpenAI, etc.) is a one-file
change in `backend/app/search.py` if you want to extend this past the prototype.

## Running it

Requires Python 3.10+.

```bash
cd agentic-drive
export ANTHROPIC_API_KEY=sk-ant-...   # optional — see "Mock mode" below
./run.sh
```

Then open **http://localhost:8000**. Click **"Load demo files"** to ingest four sample
documents (three Q3 marketing meeting transcripts with a deliberate budget change
between them, plus a hosting invoice) and watch the Organizer and Workflow agents act
immediately, or drag in your own files.

### Try this demo script (matches the brief's examples)

1. **Load demo files** and watch the Activity Feed: the Organizer Agent files the three
   meeting transcripts into the same cluster and explains why, and the Workflow Agent
   fires on the invoice and logs it to the expense table on the right, unprompted.
2. Ask: *"What was the budget discussed in the Q3 marketing meetings?"* → the Retrieval
   Agent answers from all three transcripts with citations, and (in live mode) calls out
   that the budget changed from $50,000 to $42,000 between meetings instead of picking
   one number silently.
3. Check the two budget-meeting files in the left pane, then type: *"Draft a one-page
   Q4 planning brief based on these"* → the Synthesis Agent reads both, drafts a new
   markdown file, and it appears back in the file list, tagged and clustered, with a
   provenance footer naming its sources.
4. Upload a new file whose text contains the word "invoice" and a dollar amount → the
   Workflow Agent logs it to `expenses.csv` with no prompt required; download the CSV
   from the right-hand pane.

### Mock mode

Without `ANTHROPIC_API_KEY` set, the app still runs end-to-end: each agent falls back to
a deterministic heuristic (keyword-based tagging/clustering, regex-based invoice field
extraction, raw passage retrieval instead of synthesized answers) so the full pipeline —
upload → organize → index → ask → synthesize → workflow — is demoable and testable with
no setup. The mode badge in the top bar always says which mode is active, and every mock
response says so inline. Set the key to see the actual agentic reasoning (topic-aware
clustering, cited synthesized answers, contradiction detection across sources, drafted
documents).

## API

All endpoints are under `/api`:

- `POST /api/files/upload` — multipart upload, runs the full ingestion pipeline
- `POST /api/seed` — ingest the bundled sample documents
- `GET /api/files`, `GET /api/files/{id}` — list / fetch a file (with full text)
- `GET /api/clusters` — files grouped by their auto-assigned cluster
- `POST /api/ask` — `{ "message": str, "selected_file_ids": [str] }` → routed to
  Retrieval or Synthesis
- `GET /api/activity` — the agent activity log
- `GET /api/expenses`, `GET /api/expenses.csv` — the Workflow Agent's expense log
- `GET /api/status` — whether live or mock mode is active

## What's modeled but not yet surfaced in the UI

From the brainstorm, two ideas are implemented in the data model so they're ready to
extend, but intentionally left out of the 3-minute-demo UI to keep the prototype's scope
honest:

- **Provenance graph** — every synthesized file stores `source_file_ids`; there's no
  "what depends on this file" view yet.
- **Staleness detection** — nothing currently re-checks a synthesis's sources after the
  fact and flags it as possibly outdated.

## Project structure

```
agentic-drive/
├── BRAINSTORM.md          # conceptual vision + agentic user-flows (read this first)
├── README.md
├── run.sh
├── backend/
│   ├── requirements.txt
│   ├── seed_data/          # sample meeting transcripts + invoice for the demo
│   └── app/
│       ├── main.py         # FastAPI routes
│       ├── config.py       # paths + live/mock mode switch
│       ├── db.py           # SQLite layer
│       ├── storage.py      # file save + text extraction (txt/md/csv/pdf)
│       ├── search.py       # TF-IDF retrieval
│       ├── llm.py          # Claude API wrapper
│       └── agents/
│           ├── organizer.py
│           ├── retrieval.py
│           ├── synthesis.py
│           ├── workflow.py
│           └── orchestrator.py
└── frontend/                # static HTML/CSS/JS UI, served by FastAPI
```
