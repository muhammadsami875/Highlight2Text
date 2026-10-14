const state = {
  files: [],
  clusters: [],
  filesById: new Map(),
  selectedFileIds: new Set(),
  view: { type: "all" }, // {type:'all'} | {type:'cluster', name} | {type:'expenses'}
  viewMode: "grid",
  detailsOpen: false,
  detailsTab: "details",
  focusedFileId: null,
  filterText: "",
};

const $ = (sel) => document.querySelector(sel);
const $$ = (sel) => Array.from(document.querySelectorAll(sel));

async function api(path, options = {}) {
  const res = await fetch(path, options);
  if (!res.ok) throw new Error(`${res.status}: ${await res.text()}`);
  return res.headers.get("content-type")?.includes("application/json") ? res.json() : res.text();
}

function escapeHtml(str) {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

function timeAgo(ts) {
  const diff = Date.now() / 1000 - ts;
  if (diff < 60) return `${Math.floor(diff)}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

const TYPE_COLORS = {
  invoice: "#d93025",
  meeting_transcript: "#1a73e8",
  spreadsheet: "#188038",
  synthesis: "#8430ce",
  other: "#5f6368",
};

function fileTypeIcon(docType, size = 36) {
  const color = TYPE_COLORS[docType] || TYPE_COLORS.other;
  const symbol =
    docType === "synthesis"
      ? '<path d="M12 5.5l1.3 4h4.2l-3.4 2.5 1.3 4L12 13.5 8.6 16l1.3-4L6.5 9.5h4.2z" fill="#fff"/>'
      : docType === "spreadsheet"
      ? '<path d="M6.5 6.5h11v11h-11z" fill="none" stroke="#fff" stroke-width="1.1"/><path d="M6.5 10.3h11M6.5 14h11M10.5 6.5v11M14.5 6.5v11" stroke="#fff" stroke-width="1"/>'
      : '<path d="M7.5 7h9M7.5 11h9M7.5 15h6" stroke="#fff" stroke-width="1.3" stroke-linecap="round"/>';
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" class="file-type-icon"><rect width="24" height="24" rx="6" fill="${color}"/>${symbol}</svg>`;
}

/* ---------------- data loading ---------------- */

async function refreshStatus() {
  const status = await api("/api/status");
  const badge = $("#mode-badge");
  if (status.live_mode) {
    badge.textContent = `live · ${status.model}`;
    badge.className = "mode-pill live";
  } else {
    badge.textContent = "mock mode — add ANTHROPIC_API_KEY";
    badge.className = "mode-pill mock";
  }
}

async function loadAll() {
  const [clusters, activity] = await Promise.all([api("/api/clusters"), api("/api/activity")]);
  state.clusters = clusters;
  state.files = clusters.flatMap((g) => g.files);
  state.filesById = new Map(state.files.map((f) => [f.id, f]));
  state.activity = activity;
  renderSidebarClusters();
  renderStorageMeter();
  renderFileArea();
  if (state.detailsOpen) renderDetailsPanel();
}

/* ---------------- sidebar ---------------- */

function renderSidebarClusters() {
  const nav = $("#cluster-nav");
  nav.innerHTML = "";
  for (const group of state.clusters) {
    const btn = document.createElement("button");
    btn.className = "nav-item";
    const active = state.view.type === "cluster" && state.view.name === group.cluster;
    if (active) btn.classList.add("active");
    btn.innerHTML = `<span class="cluster-dot"></span><span>${escapeHtml(group.cluster)}</span><span class="count">${group.files.length}</span>`;
    btn.addEventListener("click", () => setView({ type: "cluster", name: group.cluster }));
    nav.appendChild(btn);
  }
}

function renderStorageMeter() {
  const total = state.files.length;
  const label = $("#storage-label");
  const fill = $("#storage-fill");
  if (total === 0) {
    label.textContent = "No files yet";
    fill.style.width = "0%";
    return;
  }
  label.textContent = `${total} file${total === 1 ? "" : "s"} organized automatically across ${state.clusters.length} workspace${state.clusters.length === 1 ? "" : "s"}`;
  fill.style.width = `${Math.min(100, total * 8)}%`;
}

function setView(view) {
  state.view = view;
  state.filterText = "";
  $("#search-input").value = "";
  $$(".nav-item").forEach((el) => el.classList.remove("active"));
  if (view.type === "all") $("#nav-all").classList.add("active");
  else if (view.type === "expenses") $("#nav-expenses").classList.add("active");
  renderSidebarClusters();
  $("#view-title").textContent = view.type === "all" ? "My Drive" : view.type === "expenses" ? "Expenses" : view.name;
  $("#toolbar").classList.toggle("hidden", view.type === "expenses");
  renderSelectionBar();
  renderFileArea();
}

/* ---------------- file area ---------------- */

function currentFiles() {
  let files = state.files;
  if (state.view.type === "cluster") files = files.filter((f) => f.cluster === state.view.name);
  if (state.filterText) {
    const q = state.filterText.toLowerCase();
    files = files.filter(
      (f) => f.filename.toLowerCase().includes(q) || f.tags.some((t) => t.toLowerCase().includes(q)) || f.summary.toLowerCase().includes(q)
    );
  }
  return files;
}

function renderFileArea() {
  const area = $("#file-area");
  if (state.view.type === "expenses") {
    renderExpenses(area);
    return;
  }
  const files = currentFiles();
  if (files.length === 0) {
    area.innerHTML = `<div class="empty-state">No files here yet. Upload one, or click "Load demo files" in the New menu.</div>`;
    return;
  }
  area.innerHTML = "";
  area.appendChild(state.viewMode === "grid" ? renderGrid(files) : renderList(files));
  updateSelectionVisuals();
}

function renderGrid(files) {
  const grid = document.createElement("div");
  grid.className = "file-grid";
  for (const f of files) {
    const card = document.createElement("div");
    card.className = "file-card";
    card.dataset.fileId = f.id;
    card.innerHTML = `
      <div class="file-card-head">
        <input type="checkbox" class="card-checkbox" ${state.selectedFileIds.has(f.id) ? "checked" : ""} />
        ${fileTypeIcon(f.doc_type, 28)}
        <span class="file-name">${escapeHtml(f.filename)}</span>
      </div>
      <div class="file-card-body">
        <div class="file-summary">${escapeHtml(f.summary)}</div>
        <div class="tag-row">${f.tags.slice(0, 3).map((t) => `<span class="tag">${escapeHtml(t)}</span>`).join("")}</div>
      </div>`;
    wireFileNode(card, f);
    grid.appendChild(card);
  }
  return grid;
}

function renderList(files) {
  const wrap = document.createElement("div");
  wrap.className = "file-list";
  wrap.innerHTML = `<div class="file-list-head"><span></span><span>Name</span><span>Workspace</span><span>Filed</span></div>`;
  for (const f of files) {
    const row = document.createElement("div");
    row.className = "file-row";
    row.dataset.fileId = f.id;
    row.innerHTML = `
      <input type="checkbox" class="row-checkbox" ${state.selectedFileIds.has(f.id) ? "checked" : ""} />
      <div class="row-name">${fileTypeIcon(f.doc_type, 24)}<span>${escapeHtml(f.filename)}</span></div>
      <div class="row-cluster">${escapeHtml(f.cluster)}</div>
      <div class="row-time">${timeAgo(f.created_at)}</div>`;
    wireFileNode(row, f);
    wrap.appendChild(row);
  }
  return wrap;
}

function wireFileNode(node, f) {
  const checkbox = node.querySelector("input[type=checkbox]");
  checkbox.addEventListener("click", (e) => e.stopPropagation());
  checkbox.addEventListener("change", (e) => toggleSelect(f.id, e.target.checked));
  node.addEventListener("click", () => {
    state.focusedFileId = f.id;
    if (state.detailsOpen) {
      state.detailsTab = "details";
      renderDetailsPanel();
    } else {
      openFilePreview(f.id);
    }
  });
}

function updateSelectionVisuals() {
  $$(".file-card").forEach((el) => el.classList.toggle("selected", state.selectedFileIds.has(el.dataset.fileId)));
  $$(".file-row").forEach((el) => el.classList.toggle("selected", state.selectedFileIds.has(el.dataset.fileId)));
}

function toggleSelect(fileId, checked) {
  if (checked) state.selectedFileIds.add(fileId);
  else state.selectedFileIds.delete(fileId);
  updateSelectionVisuals();
  renderSelectionBar();
}

function renderSelectionBar() {
  const n = state.selectedFileIds.size;
  $("#selection-bar").classList.toggle("hidden", n === 0 || state.view.type === "expenses");
  $("#selection-count").textContent = `${n} selected`;
}

/* ---------------- expenses view ---------------- */

async function renderExpenses(area) {
  area.innerHTML = `<div class="expenses-toolbar"><a class="pill-btn" href="/api/expenses.csv">Download CSV</a></div>
    <table class="expenses-table"><thead><tr><th>Date</th><th>Vendor</th><th>Amount</th><th>Category</th><th>Source file</th></tr></thead><tbody id="expenses-tbody"></tbody></table>`;
  const rows = await api("/api/expenses");
  const tbody = $("#expenses-tbody");
  if (rows.length === 0) {
    tbody.innerHTML = `<tr><td colspan="5" style="color:var(--text-secondary)">No invoices logged yet — upload one and the Workflow Agent will log it here automatically.</td></tr>`;
    return;
  }
  tbody.innerHTML = rows
    .map(
      (r) => `<tr><td>${r.date}</td><td>${escapeHtml(r.vendor)}</td><td>$${Number(r.amount).toFixed(2)}</td><td>${escapeHtml(r.category)}</td><td>${escapeHtml(r.source_file)}</td></tr>`
    )
    .join("");
}

/* ---------------- details / activity panel ---------------- */

function toggleDetailsPanel() {
  state.detailsOpen = !state.detailsOpen;
  $("#details-panel").classList.toggle("hidden", !state.detailsOpen);
  $("#details-toggle").classList.toggle("active", state.detailsOpen);
  if (state.detailsOpen) renderDetailsPanel();
}

function renderDetailsPanel() {
  $$(".details-tabs .tab").forEach((t) => t.classList.toggle("active", t.dataset.tab === state.detailsTab));
  const body = $("#details-body");
  if (state.detailsTab === "activity") {
    body.innerHTML = `<div class="activity-feed">${
      state.activity.length === 0
        ? `<div class="details-empty">No activity yet.</div>`
        : state.activity
            .map((a) => `<div class="activity-item"><span class="agent-tag">${escapeHtml(a.agent)}</span>${escapeHtml(a.message)}<span class="ts">${timeAgo(a.ts)}</span></div>`)
            .join("")
    }</div>`;
    return;
  }

  const f = state.focusedFileId ? state.filesById.get(state.focusedFileId) : null;
  if (!f) {
    body.innerHTML = `<div class="details-empty">Select a file to see what the agents did with it.</div>`;
    return;
  }
  const sources = (f.source_file_ids || []).map((id) => state.filesById.get(id)).filter(Boolean);
  body.innerHTML = `
    <div class="details-icon-big">${fileTypeIcon(f.doc_type, 56)}</div>
    <div class="details-filename">${escapeHtml(f.filename)}</div>
    <div class="details-meta-row"><span class="label">Workspace</span><span>${escapeHtml(f.cluster)}</span></div>
    <div class="details-meta-row"><span class="label">Type</span><span>${escapeHtml(f.doc_type.replace("_", " "))}</span></div>
    <div class="details-meta-row"><span class="label">Filed</span><span>${timeAgo(f.created_at)}</span></div>
    <div class="details-section-title">Why it's filed here</div>
    <div class="details-reason">${escapeHtml(f.cluster_reason || "—")}</div>
    <div class="details-section-title">Summary</div>
    <div class="file-summary">${escapeHtml(f.summary)}</div>
    <div class="details-section-title">Tags</div>
    <div class="tag-row">${f.tags.map((t) => `<span class="tag">${escapeHtml(t)}</span>`).join("")}</div>
    ${
      sources.length
        ? `<div class="details-section-title">Generated from</div>${sources
            .map((s) => `<div class="details-source-link" data-file-id="${s.id}">${escapeHtml(s.filename)}</div>`)
            .join("")}`
        : ""
    }
    <div class="details-section-title">Open</div>
    <div class="details-source-link" data-file-id="${f.id}" data-preview="1">View full text</div>
  `;
  $$(".details-source-link", body).forEach((el) =>
    el.addEventListener("click", () => {
      if (el.dataset.preview) openFilePreview(el.dataset.fileId);
      else {
        state.focusedFileId = el.dataset.fileId;
        renderDetailsPanel();
      }
    })
  );
}

$$(".details-tabs .tab").forEach((tab) =>
  tab.addEventListener("click", () => {
    state.detailsTab = tab.dataset.tab;
    renderDetailsPanel();
  })
);

/* ---------------- file preview modal ---------------- */

async function openFilePreview(fileId) {
  const f = await api(`/api/files/${fileId}`);
  $("#modal-title").textContent = f.filename;
  $("#modal-body").textContent = f.text;
  $("#file-modal").classList.remove("hidden");
}

$$("[data-close-modal]").forEach((btn) => btn.addEventListener("click", () => $(`#${btn.dataset.closeModal}`).classList.add("hidden")));
$$(".modal").forEach((modal) => modal.addEventListener("click", (e) => { if (e.target === modal) modal.classList.add("hidden"); }));

/* ---------------- search / ask ---------------- */

const searchInput = $("#search-input");
searchInput.addEventListener("input", () => {
  state.filterText = searchInput.value.trim();
  $("#search-clear").classList.toggle("hidden", !state.filterText);
  $("#search-dropdown").classList.add("hidden");
  if (state.view.type !== "expenses") renderFileArea();
});

$("#search-clear").addEventListener("click", () => {
  searchInput.value = "";
  state.filterText = "";
  $("#search-clear").classList.add("hidden");
  $("#search-dropdown").classList.add("hidden");
  renderFileArea();
});

searchInput.addEventListener("keydown", async (e) => {
  if (e.key !== "Enter") return;
  const message = searchInput.value.trim();
  if (!message) return;
  const dropdown = $("#search-dropdown");
  dropdown.classList.remove("hidden");
  dropdown.innerHTML = `<div class="loading">Asking the agents…</div>`;
  try {
    const result = await api("/api/ask", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ message, selected_file_ids: Array.from(state.selectedFileIds) }),
    });
    const citations = (result.citations || [])
      .map((c) => `<button class="citation-chip" data-filename="${escapeHtml(c)}">${escapeHtml(c)}</button>`)
      .join("");
    dropdown.innerHTML = `<span class="agent-label">${result.agent} agent</span><div class="answer-text">${escapeHtml(result.answer)}</div>${citations}`;
    $$(".citation-chip", dropdown).forEach((chip) =>
      chip.addEventListener("click", () => {
        const match = state.files.find((f) => f.filename === chip.dataset.filename);
        if (match) openFilePreview(match.id);
      })
    );
    if (result.generated_file_id) await loadAll();
    else state.activity = await api("/api/activity");
  } catch (err) {
    dropdown.innerHTML = `<span class="agent-label">error</span>${escapeHtml(err.message)}`;
  }
});

document.addEventListener("click", (e) => {
  if (!$("#search-pill").contains(e.target) && !$("#search-dropdown").contains(e.target)) {
    $("#search-dropdown").classList.add("hidden");
  }
});

/* ---------------- new menu / upload / seed ---------------- */

$("#new-btn").addEventListener("click", (e) => {
  e.stopPropagation();
  $("#new-menu").classList.toggle("hidden");
});
document.addEventListener("click", (e) => {
  if (!$("#new-btn").contains(e.target) && !$("#new-menu").contains(e.target)) $("#new-menu").classList.add("hidden");
});
$$("#new-menu button").forEach((btn) =>
  btn.addEventListener("click", () => {
    $("#new-menu").classList.add("hidden");
    if (btn.dataset.action === "upload") $("#file-input").click();
    else runSeed();
  })
);

$("#file-input").addEventListener("change", async (e) => {
  const file = e.target.files[0];
  if (!file) return;
  const form = new FormData();
  form.append("file", file);
  try {
    await api("/api/files/upload", { method: "POST", body: form });
    await loadAll();
  } catch (err) {
    alert(`Upload failed: ${err.message}`);
  }
  e.target.value = "";
});

async function runSeed() {
  const result = await api("/api/seed", { method: "POST" });
  await loadAll();
  if (result.ingested.length === 0) alert("Demo files are already loaded.");
}

/* ---------------- selection bar actions ---------------- */

$("#selection-clear").addEventListener("click", () => {
  state.selectedFileIds.clear();
  updateSelectionVisuals();
  renderSelectionBar();
});

$("#ask-selected-btn").addEventListener("click", () => {
  searchInput.focus();
  searchInput.value = "What do these files say about ";
  searchInput.setSelectionRange(searchInput.value.length, searchInput.value.length);
});

$("#draft-selected-btn").addEventListener("click", () => {
  $("#prompt-count").textContent = state.selectedFileIds.size;
  $("#prompt-input").value = "";
  $("#prompt-modal").classList.remove("hidden");
  $("#prompt-input").focus();
});

$("#prompt-submit").addEventListener("click", submitDraft);
$("#prompt-input").addEventListener("keydown", (e) => { if (e.key === "Enter") submitDraft(); });

async function submitDraft() {
  const message = $("#prompt-input").value.trim();
  if (!message) return;
  $("#prompt-modal").classList.add("hidden");
  try {
    await api("/api/ask", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ message, selected_file_ids: Array.from(state.selectedFileIds) }),
    });
    state.selectedFileIds.clear();
    renderSelectionBar();
    await loadAll();
  } catch (err) {
    alert(`Synthesis failed: ${err.message}`);
  }
}

/* ---------------- toolbar: nav, view toggle, details ---------------- */

$("#nav-all").addEventListener("click", () => setView({ type: "all" }));
$("#nav-expenses").addEventListener("click", () => setView({ type: "expenses" }));

$("#view-grid-btn").addEventListener("click", () => setViewMode("grid"));
$("#view-list-btn").addEventListener("click", () => setViewMode("list"));
function setViewMode(mode) {
  state.viewMode = mode;
  $("#view-grid-btn").classList.toggle("active", mode === "grid");
  $("#view-list-btn").classList.toggle("active", mode === "list");
  renderFileArea();
}

$("#details-toggle").addEventListener("click", toggleDetailsPanel);
$("#activity-toggle").addEventListener("click", () => {
  if (!state.detailsOpen) toggleDetailsPanel();
  state.detailsTab = "activity";
  renderDetailsPanel();
});

/* ---------------- boot ---------------- */

refreshStatus();
loadAll();
setInterval(async () => {
  state.activity = await api("/api/activity");
  if (state.detailsOpen && state.detailsTab === "activity") renderDetailsPanel();
}, 8000);
