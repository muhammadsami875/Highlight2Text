const state = {
  selectedFileIds: new Set(),
  filesById: new Map(),
};

const $ = (sel) => document.querySelector(sel);

async function api(path, options = {}) {
  const res = await fetch(path, options);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status}: ${text}`);
  }
  return res.headers.get("content-type")?.includes("application/json") ? res.json() : res.text();
}

function docTypeIcon(docType) {
  return { invoice: "🧾", meeting_transcript: "🗒️", spreadsheet: "📊", synthesis: "✨" }[docType] || "📄";
}

function timeAgo(ts) {
  const diff = Date.now() / 1000 - ts;
  if (diff < 60) return `${Math.floor(diff)}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  return `${Math.floor(diff / 3600)}h ago`;
}

async function refreshStatus() {
  const status = await api("/api/status");
  const badge = $("#mode-badge");
  if (status.live_mode) {
    badge.textContent = `live · ${status.model}`;
    badge.className = "badge live";
  } else {
    badge.textContent = "offline / mock mode — add ANTHROPIC_API_KEY";
    badge.className = "badge mock";
  }
}

async function refreshClusters() {
  const clusters = await api("/api/clusters");
  const container = $("#clusters");
  container.innerHTML = "";
  state.filesById.clear();

  if (clusters.length === 0) {
    container.innerHTML = `<p class="hint">No files yet. Upload one or click "Load demo files".</p>`;
  }

  for (const group of clusters) {
    const groupEl = document.createElement("div");
    groupEl.className = "cluster-group";
    groupEl.innerHTML = `<div class="cluster-name">${group.cluster} <span class="count">${group.files.length}</span></div>`;

    for (const f of group.files) {
      state.filesById.set(f.id, f);
      const card = document.createElement("div");
      card.className = "file-card";
      card.title = f.cluster_reason || "";
      card.innerHTML = `
        <div class="file-row">
          <input type="checkbox" data-file-id="${f.id}" ${state.selectedFileIds.has(f.id) ? "checked" : ""} />
          <div>
            <div class="file-name">${docTypeIcon(f.doc_type)} ${f.filename}</div>
            <div class="file-summary">${f.summary}</div>
            <div>${f.tags.map((t) => `<span class="tag">${t}</span>`).join("")}</div>
          </div>
        </div>`;
      card.querySelector("input").addEventListener("change", (e) => {
        if (e.target.checked) state.selectedFileIds.add(f.id);
        else state.selectedFileIds.delete(f.id);
        renderSelectionBar();
      });
      card.querySelector(".file-name").addEventListener("click", () => openFileModal(f.id));
      groupEl.appendChild(card);
    }
    container.appendChild(groupEl);
  }
}

function renderSelectionBar() {
  const bar = $("#selection-bar");
  const n = state.selectedFileIds.size;
  bar.textContent = n > 0
    ? `${n} file(s) selected for synthesis — try "draft a summary from these"`
    : "";
}

async function openFileModal(fileId) {
  const f = await api(`/api/files/${fileId}`);
  $("#modal-title").textContent = f.filename;
  $("#modal-body").textContent = f.text;
  $("#file-modal").classList.remove("hidden");
}

function closeModal() {
  $("#file-modal").classList.add("hidden");
}

async function refreshActivity() {
  const items = await api("/api/activity");
  const feed = $("#activity-feed");
  feed.innerHTML = "";
  if (items.length === 0) {
    feed.innerHTML = `<p class="hint">No activity yet.</p>`;
  }
  for (const item of items) {
    const el = document.createElement("div");
    el.className = "activity-item";
    el.innerHTML = `<span class="agent-tag">${item.agent}</span>${item.message}<span class="ts">${timeAgo(item.ts)}</span>`;
    feed.appendChild(el);
  }
}

async function refreshExpenses() {
  const rows = await api("/api/expenses");
  const tbody = $("#expenses-table tbody");
  tbody.innerHTML = "";
  for (const r of rows) {
    const tr = document.createElement("tr");
    tr.innerHTML = `<td>${r.date}</td><td>${r.vendor}</td><td>$${Number(r.amount).toFixed(2)}</td><td>${r.category}</td>`;
    tbody.appendChild(tr);
  }
}

function appendMessage(role, html) {
  const log = $("#chat-log");
  const el = document.createElement("div");
  el.className = `msg ${role}`;
  el.innerHTML = html;
  log.appendChild(el);
  log.scrollTop = log.scrollHeight;
}

async function refreshAll() {
  await Promise.all([refreshClusters(), refreshActivity(), refreshExpenses()]);
}

$("#chat-form").addEventListener("submit", async (e) => {
  e.preventDefault();
  const input = $("#chat-input");
  const message = input.value.trim();
  if (!message) return;
  appendMessage("user", message);
  input.value = "";

  try {
    const result = await api("/api/ask", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ message, selected_file_ids: Array.from(state.selectedFileIds) }),
    });
    const citations = (result.citations || [])
      .map((c) => `<span class="citation-chip" data-filename="${c}">${c}</span>`)
      .join("");
    appendMessage(
      "agent",
      `<span class="agent-label">${result.agent} agent</span>${escapeHtml(result.answer)}${citations ? `<div>${citations}</div>` : ""}`
    );
    for (const chip of document.querySelectorAll("#chat-log .msg:last-child .citation-chip")) {
      chip.addEventListener("click", () => {
        const match = Array.from(state.filesById.values()).find((f) => f.filename === chip.dataset.filename);
        if (match) openFileModal(match.id);
      });
    }
    if (result.generated_file_id) {
      await refreshAll();
    } else {
      await refreshActivity();
    }
  } catch (err) {
    appendMessage("agent", `<span class="agent-label">error</span>${escapeHtml(err.message)}`);
  }
});

function escapeHtml(str) {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

$("#file-input").addEventListener("change", async (e) => {
  const file = e.target.files[0];
  if (!file) return;
  const form = new FormData();
  form.append("file", file);
  appendMessage("agent", `<span class="agent-label">organizer agent</span>Ingesting "${file.name}"…`);
  try {
    await api("/api/files/upload", { method: "POST", body: form });
    await refreshAll();
  } catch (err) {
    appendMessage("agent", `<span class="agent-label">error</span>${escapeHtml(err.message)}`);
  }
  e.target.value = "";
});

$("#seed-btn").addEventListener("click", async () => {
  appendMessage("agent", `<span class="agent-label">organizer agent</span>Loading demo files…`);
  const result = await api("/api/seed", { method: "POST" });
  appendMessage("agent", `<span class="agent-label">organizer agent</span>Ingested ${result.ingested.length} demo file(s).`);
  await refreshAll();
});

$("#modal-close").addEventListener("click", closeModal);
$("#file-modal").addEventListener("click", (e) => {
  if (e.target.id === "file-modal") closeModal();
});

refreshStatus();
refreshAll();
setInterval(() => { refreshActivity(); refreshExpenses(); }, 8000);
