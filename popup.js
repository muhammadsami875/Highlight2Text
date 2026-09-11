const listEl = document.getElementById("list");
const grabBtn = document.getElementById("grab");
const downloadBtn = document.getElementById("download");
const copyBtn = document.getElementById("copy");
const clearBtn = document.getElementById("clear");

const SEP = "\n\n----------\n\n";

async function getSnippets() {
  const { snippets = [] } = await chrome.storage.local.get("snippets");
  return snippets;
}

async function setSnippets(snippets) {
  await chrome.storage.local.set({ snippets });
  chrome.action.setBadgeText({ text: snippets.length ? String(snippets.length) : "" });
  render(snippets);
}

function render(snippets) {
  listEl.innerHTML = "";
  const has = snippets.length > 0;
  downloadBtn.disabled = !has;
  copyBtn.disabled = !has;
  clearBtn.disabled = !has;

  if (!has) {
    const p = document.createElement("p");
    p.className = "empty";
    p.textContent =
      "No text saved yet. Highlight text on any page and use the “Save .txt” button that pops up, the right-click menu, or “Grab current selection” above.";
    listEl.appendChild(p);
    return;
  }

  snippets.forEach((s, i) => {
    const item = document.createElement("div");
    item.className = "item";

    const p = document.createElement("p");
    p.className = "text";
    p.textContent = s.text;

    const del = document.createElement("button");
    del.className = "del";
    del.textContent = "✕";
    del.title = "Remove";
    del.addEventListener("click", async () => {
      const arr = await getSnippets();
      arr.splice(i, 1);
      setSnippets(arr);
    });

    item.appendChild(p);
    item.appendChild(del);
    listEl.appendChild(item);
  });
}

grabBtn.addEventListener("click", async () => {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab || !tab.id) return;

  let text = "";
  try {
    const [res] = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: () => window.getSelection().toString()
    });
    text = ((res && res.result) || "").trim();
  } catch (e) {
    text = "";
  }

  if (!text) {
    flash(grabBtn, "No text selected!", "Grab current selection");
    return;
  }
  const arr = await getSnippets();
  arr.push({ text, at: Date.now() });
  setSnippets(arr);
});

downloadBtn.addEventListener("click", async () => {
  const arr = await getSnippets();
  const body = arr.map((s) => s.text).join(SEP);
  const dataUrl = "data:text/plain;charset=utf-8," + encodeURIComponent(body);
  const stamp = new Date().toISOString().slice(0, 10);
  chrome.downloads.download({
    url: dataUrl,
    filename: `text-grabber-${stamp}.txt`,
    saveAs: true
  });
});

copyBtn.addEventListener("click", async () => {
  const arr = await getSnippets();
  await navigator.clipboard.writeText(arr.map((s) => s.text).join(SEP));
  flash(copyBtn, "Copied!", "Copy all");
});

clearBtn.addEventListener("click", () => setSnippets([]));

function flash(el, temp, original) {
  el.textContent = temp;
  setTimeout(() => (el.textContent = original), 1100);
}

getSnippets().then(render);
