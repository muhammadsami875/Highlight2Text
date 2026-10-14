# Agentic Drive

A prototype "Advanced Google Drive" — an active workspace where autonomous agents
organize, explain, synthesize, and act on your files instead of just storing them. The
UI is deliberately styled after Google Drive itself (search-first top bar, "+ New"
button, sidebar nav, grid/list toggle, a details side panel) — not to imitate Google's
product, but because that chrome is the fastest way to show judges "this is a drive,"
freeing the actual demo time for the agent behavior that's different from it: the
search bar answers questions instead of just filtering, the details panel explains a
filing decision instead of showing file size, and selecting files surfaces a "Draft
from these" action no file browser has.

See **[BRAINSTORM.md](./BRAINSTORM.md)** for the conceptual vision and the agentic
user-flows this was designed from before any code was written.

## What it does

Every file that enters the drive goes through one ingestion pipeline, and four small
agents sit on top of it. Every agent is **rule-based** — keyword/pattern matching and
TF-IDF ranking, no external LLM API call and nothing to configure:

| Agent | Brief's capability | What it does |
|---|---|---|
| **Organizer** | Proactive Organization | Tags every file, assigns it to an existing cluster or starts a new one by keyword overlap, and writes a one-line explanation of *why* — no manual folders, ever. |
| **Retrieval** | Semantic Retrieval | Answers free-text questions ("What was the budget discussed in the Q3 marketing meetings?") by TF-IDF-ranking the most relevant passages across every file and citing sources inline. |
| **Synthesis** | Cross-Document Synthesis | Reads several selected files and assembles a new document from an instruction (e.g. "draft a unified PRD from these") by merging their summaries and leading content, then **saves the result back into the drive** as a first-class, tagged, provenance-stamped file. |
| **Workflow** | Automated Workflows | Triggers automatically (no prompt) when an uploaded file looks like an invoice: extracts vendor/amount/date/category via pattern matching and logs a row to the expense log. |

A thin **Orchestrator** routes the chat box's free text to Retrieval or Synthesis
depending on whether you've selected files and whether the message reads like a
drafting instruction — this is the "Routing & Synthesis" middleware layer from the
brief's diagram.

Every agent action — a filing decision, a synthesis, an expense logged — is written to
an **Activity Feed** so autonomous behavior stays legible instead of opaque.

Drive-management basics are there too, styled the same way: each file has a kebab
(⋮) menu for **Open** / **Delete**, the selection bar gets a **Delete** action for
bulk removal, and the avatar in the top-right opens an account popover with a real
Drive-style storage meter ("X of 16 GB used," computed from actual stored content,
not a placeholder) and a "Reset drive" action that clears everything. There's no real
authentication behind the account popover — it's a single-user demo, so the identity
shown is static rather than faked as a real login.

## Architecture

```
┌─────────────────────────────── Frontend (static HTML/JS, served from project root) ──────┐
│  Workspaces (clusters/files)   │   Ask your drive (chat)   │   Agent activity + expenses  │
└───────────────┬─────────────────────────────┬──────────────────────────┬──────────────────┘
                 │ upload                      │ ask / synthesize         │ poll
                 ▼                             ▼                          ▼
┌───────────────────────────────── FastAPI (backend/app) ──────────────────────────────────┐
│  Ingestion pipeline:  extract text (in-memory) → Organizer Agent → index → Workflow Agent  │
│  Chat pipeline:       Orchestrator → Retrieval Agent  |  Synthesis Agent (writes back)     │
│  Storage:             Postgres (DATABASE_URL) when set, else local SQLite — files, tags,   │
│                       clusters, activity log, and the expense log all live in the DB, not  │
│                       on disk, so the app has no persistent-filesystem dependency          │
│  Agents:              rule-based throughout — TF-IDF ranking for retrieval, keyword        │
│                       overlap for clustering, regex for invoice field extraction           │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

This runs equally well two ways, with no code changes between them:
- **Locally** via `./run.sh` — SQLite file on disk, FastAPI serves the frontend itself.
- **On Vercel** as a Python serverless function (`api/index.py`) — the frontend is served
  directly by Vercel's CDN from the project root, and the backend talks to Postgres since
  a serverless function's filesystem is read-only outside of `/tmp` and nothing written
  there survives past the current invocation.

**Why pure-Python TF-IDF instead of a hosted vector DB or an LLM:** no external API
calls means no API key, no billing, no network dependency, and no risk of a demo
failing on an unfamiliar machine or because of a third-party outage — every agent runs
entirely on the drive's own content. TF-IDF retrieval (no dependencies at all — see
`backend/app/search.py`) gets the same user-facing shape ("ask anything, get a cited
answer") without any of that.

## Running it locally

Requires Python 3.10+.

```bash
cd agentic-drive
./run.sh
```

Then open **http://localhost:8000**. Click **"Load demo files"** to ingest four sample
documents (three Q3 marketing meeting transcripts with a deliberate budget change
between them, plus a hosting invoice) and watch the Organizer and Workflow agents act
immediately, or drag in your own files.

## Deploying to Vercel

```bash
npm i -g vercel     # if you don't already have the CLI
cd agentic-drive
vercel              # first deploy — follow the prompts, accept defaults
```

Vercel auto-detects `api/index.py` (a FastAPI app) as a Python serverless function,
installs the root `requirements.txt`, and serves `index.html` / `app.js` / `style.css`
directly from the project root as static assets. `vercel.json` rewrites `/api/*`
requests to the function; everything else is static.

The only environment variable worth setting is persistence:

| Variable | Required? | What it does |
|---|---|---|
| `DATABASE_URL` (or `POSTGRES_URL`) | **Strongly recommended** | A Postgres connection string. **Without this, the app falls back to a SQLite file under `/tmp`, which is wiped on every cold start** — uploaded files and the activity/expense log will appear to vanish between requests. The easiest path: in the Vercel dashboard, go to **Storage → Create Database → Postgres** and connect it to this project; Vercel sets `POSTGRES_URL` for you automatically, no code changes needed. |

After adding it, redeploy (`vercel --prod`) so the function picks it up.

**If Vercel's import UI says "Multiple applications detected" and prompts you to set
up "Services":** that's a false positive, not a sign you need the multi-app monorepo
flow — ignore it and look for a way to import normally (or, importing from GitHub,
make sure there's only one `requirements.txt` in the repo; this project keeps a single
one at the root specifically to avoid tripping that detector). There is exactly one
backend here.

**If the root URL returns `{"detail":"Not Found"}` instead of the UI:** Vercel likely
auto-detected the Framework Preset as "FastAPI" and is routing every path through the
function instead of serving the static files. Project → **Settings → General →
Framework Preset → Other**, save, then redeploy.

### Try this demo script (matches the brief's examples)

1. **Load demo files** and watch the Activity Feed: the Organizer Agent files the three
   meeting transcripts into the same cluster and explains why, and the Workflow Agent
   fires on the invoice and logs it to the expense table on the right, unprompted.
2. Ask: *"What was the budget discussed in the Q3 marketing meetings?"* → the Retrieval
   Agent returns the most relevant passages from all three transcripts with citations —
   including the kickoff meeting's $50,000 figure and the later meeting's revised
   $42,000, so you can see the discrepancy yourself in the sources.
3. Check the two budget-meeting files in the left pane, then type: *"Draft a one-page
   Q4 planning brief based on these"* → the Synthesis Agent merges both, drafts a new
   markdown file, and it appears back in the file list, tagged and clustered, with a
   provenance footer naming its sources.
4. Upload a new file whose text contains the word "invoice" and a dollar amount → the
   Workflow Agent logs it to the expense log with no prompt required; download it as
   CSV from the right-hand pane.

## API

All endpoints are under `/api`:

- `POST /api/files/upload` — multipart upload, runs the full ingestion pipeline
- `POST /api/seed` — ingest the bundled sample documents
- `GET /api/files`, `GET /api/files/{id}` — list / fetch a file (with full text)
- `GET /api/clusters` — files grouped by their auto-assigned cluster
- `POST /api/ask` — `{ "message": str, "selected_file_ids": [str] }` → routed to
  Retrieval or Synthesis
- `DELETE /api/files/{id}` — delete one file
- `POST /api/files/delete-bulk` — `{ "file_ids": [str] }` → delete several at once
- `DELETE /api/files` — delete every file ("reset drive")
- `GET /api/storage` — `{ used_bytes, file_count, quota_bytes }` for the storage meter
- `GET /api/activity` — the agent activity log
- `GET /api/expenses`, `GET /api/expenses.csv` — the Workflow Agent's expense log
- `GET /api/status` — which storage backend is active (`postgres` or `sqlite`)

## What's modeled but not yet surfaced in the UI

From the brainstorm, two ideas are implemented in the data model so they're ready to
extend, but intentionally left out of the 3-minute-demo UI to keep the prototype's scope
honest:

- **Provenance graph** — every synthesized file stores `source_file_ids`; there's no
  "what depends on this file" view yet.
- **Staleness detection** — nothing currently re-checks a synthesis's sources after the
  fact and flags it as possibly outdated.

Also worth noting: the rule-based agents here are a deliberate scope choice for this
prototype (see "Why pure-Python TF-IDF instead of a hosted vector DB or an LLM" above),
not a limitation of the architecture — BRAINSTORM.md's vision assumes real language
understanding, and each agent's module is small and self-contained enough that swapping
in an LLM call (Claude or otherwise) for any one of them is a localized change, not a
rewrite.

## Project structure

```
agentic-drive/
├── BRAINSTORM.md          # conceptual vision + agentic user-flows (read this first)
├── README.md
├── run.sh                  # local dev launcher (installs requirements.txt + uvicorn)
├── vercel.json             # rewrites /api/* to the serverless function
├── requirements.txt        # the ONLY requirements.txt in the repo — deps for both the
│                             Vercel function and local dev. Deliberately not duplicated
│                             under backend/: a second requirements.txt there makes
│                             Vercel's import scanner think this is a multi-app monorepo
│                             and push you into its "Services" flow, which this project
│                             doesn't need (it's one backend, not several).
├── index.html, app.js, style.css   # static frontend — served by Vercel's CDN in prod,
│                                      or by FastAPI itself when running via run.sh
├── api/
│   └── index.py            # Vercel entrypoint — re-exports the FastAPI app below
└── backend/
    ├── seed_data/          # sample meeting transcripts + invoice for the demo
    └── app/
        ├── main.py         # FastAPI routes
        ├── config.py       # paths + sqlite/postgres switch
        ├── db.py           # storage layer (Postgres via DATABASE_URL, else SQLite)
        ├── storage.py      # in-memory text extraction (txt/md/csv/pdf) — no disk writes
        ├── search.py       # pure-Python TF-IDF retrieval, no dependencies
        └── agents/
            ├── organizer.py
            ├── retrieval.py
            ├── synthesis.py
            ├── workflow.py
            └── orchestrator.py
```
