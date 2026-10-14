# Agentic Drive — Brainstorming & Conceptual Vision

## 1. Reframing the problem

A traditional drive answers one question: *"where did I put this file?"*
An agentic drive should answer the questions people actually have:
*"what does my data know?"*, *"what should I do next?"*, *"what changed, and does it matter?"*

So instead of designing "a file browser with AI search bolted on," we designed around three
shifts in who does the work:

| Passive drive | Agentic Drive |
|---|---|
| User creates folders | Agent proposes & maintains structure, user can override |
| User searches with keywords | User asks a question, agent reads across files and answers with citations |
| User reads N files to write a summary | Agent reads N files and writes the summary, citing its sources, and files it back in the drive |
| User notices an invoice and does bookkeeping | Agent notices the invoice and does the bookkeeping |
| Drive is a destination | Drive is a collaborator that has opinions about its own contents |

The common thread: **every file write is also an event an agent can react to.** That's the
middleware layer the brief asks for — not a chatbot next to the file list, but an event bus
where upload → classify → tag → cluster → (maybe) act is the default path for *every* file,
not an opt-in feature.

## 2. Agentic user-flows we designed before writing code

### Flow A — Ingestion (every file, no exceptions)
```
Upload ──▶ Extract text ──▶ Organizer Agent
                              ├─ classify topic/type
                              ├─ generate tags
                              ├─ assign to an existing cluster OR spin up a new one
                              ├─ write a 1-line summary
                              └─ hand off to Workflow Agent (see Flow D)
                                            │
                                            ▼
                              Index chunks into the semantic store
                                            │
                                            ▼
                              Log the decision to the Activity Feed
                              ("Filed under *Q3 Marketing* because it mentions
                               the same campaign budget as 2 other files")
```
The key "aha": the agent **explains its filing decision** the way a human assistant would,
instead of silently moving a file. Trust in autonomous organization comes from legibility,
not accuracy alone.

### Flow B — Semantic retrieval ("ask the drive")
```
"What was the budget discussed in the Q3 marketing meetings?"
        │
        ▼
Retrieval Agent → searches across PDFs / transcripts / sheets
        │
        ▼
Synthesizes ONE answer, every claim carrying a [filename:chunk] citation
        │
        ▼
User clicks a citation → file opens scrolled to that passage
```

### Flow C — Cross-document synthesis (the drive writes back to itself)
```
User selects 3 transcripts → "Draft a unified PRD from these"
        │
        ▼
Synthesis Agent reads all 3, resolves overlaps/contradictions, drafts doc
        │
        ▼
New file is saved INTO the drive (not just shown in a chat bubble),
auto-tagged and clustered like any other file, with a
"Generated from: transcript_1, transcript_2, transcript_3" provenance footer
```
The "aha": output of an agent is not a chat message that evaporates — it's a first-class
citizen of the drive, with its own lineage, that can itself be an input to a future synthesis.

### Flow D — Automated workflows (trigger → act, no prompt required)
```
invoice.pdf lands in /Finance
        │
        ▼
Workflow Agent detects doc type == invoice
        │
        ▼
Extracts vendor / amount / date / category
        │
        ▼
Appends a row to expenses.csv (created on first use) + logs the action
        │
        ▼
Activity Feed: "Logged $1,240 to Acme Hosting under Infrastructure — 2s ago"
```

## 3. "Aha" ideas beyond the brief's four pillars

We deliberately brainstormed past the four listed capabilities, because the brief rewards
surprising behavior, not just coverage of the checklist:

1. **Contradiction Detector.** When two files disagree (e.g., the kickoff deck says the budget
   is $50k, the later meeting transcript says $42k), the agent doesn't just answer with
   whichever it finds first — it flags the conflict explicitly: *"Two sources disagree: deck
   (Sept 2) says $50k, transcript (Sept 20) says $42k — the transcript is more recent."*
2. **Living Index, not a snapshot.** Clusters are never finalized — adding a 4th Q3 marketing
   file can cause the agent to split a cluster that was previously one topic, the same way a
   human reorganizing folders would, and it tells you why it split them.
3. **Provenance graph.** Every generated document stores which source files produced it. This
   turns the drive into a DAG of knowledge, not a pile of files — you can ask "what depends on
   this transcript?" before you delete it.
4. **Staleness / knowledge-decay nudges.** If a generated summary's source files have since
   been edited or superseded, the agent flags the summary as "may be out of date" instead of
   silently serving stale synthesis forever.
5. **Agent explains its own filing, every time** (see Flow A) — this is a UX decision as much
   as a technical one, and it's what makes "autonomous" feel safe instead of opaque.
6. **The drive as a collaborator with memory of its own actions** — the Activity Feed is not a
   log for debugging, it's the main proof-of-work surface for the user: "here is everything the
   agents did on your behalf while you weren't watching."

## 4. Scope we chose for the prototype

Given hackathon time, we implemented the full vertical slice of all four core capabilities end
to end (ingest → tag/cluster → semantic Q&A with citations → cross-doc synthesis written back
to the drive → an invoice-triggered expense workflow), plus #1, #2, #5 and #6 from the "aha"
list above, because those are the ones that are visible in a 3-minute demo without needing a
large multi-week dataset. #3 and #4 (provenance graph, staleness nudges) are modeled in the
data (every generated file stores its `source_file_ids`) but the UI surfacing is left as the
clearly-labeled "next" step in `README.md` — we preferred a smaller set of ideas that actually
run over a longer list that's only described.

## 5. Why this architecture

- **One ingestion path, no manual folders.** There is intentionally no "new folder" button.
  That's the thesis of the brief — if the agent needs a human to create structure, it isn't
  actually agentic.
- **Agents are small and composable**, not one giant prompt: Organizer, Retrieval, Synthesis,
  and Workflow agents each do one job and are orchestrated by a thin router. This mirrors how
  we'd actually extend this into a real product — new agents (e.g., a "Contract Risk" agent)
  slot into the same ingestion event without touching the others.
- **Offline-safe by design.** Every agent has a deterministic fallback path when no LLM key is
  configured, so the system is demoable and testable without depending on a live API key during
  judging/setup (see README "Mock mode").
