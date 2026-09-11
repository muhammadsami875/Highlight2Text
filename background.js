// Runs in the background. Handles the right-click menu, downloads,
// and messages coming from the floating button and the popup.

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "download-selection",
    title: "Download selection as .txt",
    contexts: ["selection"]
  });
  chrome.contextMenus.create({
    id: "add-selection",
    title: "Add selection to collection",
    contexts: ["selection"]
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  const text = (info.selectionText || "").trim();
  if (!text) return;

  if (info.menuItemId === "download-selection") {
    downloadText(text, tab && tab.title);
  } else if (info.menuItemId === "add-selection") {
    addToCollection(text);
  }
});

// Messages from content.js (floating button) and popup.js
chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.type === "download") {
    downloadText(msg.text, msg.title);
    sendResponse({ ok: true });
  } else if (msg.type === "add") {
    addToCollection(msg.text).then(() => sendResponse({ ok: true }));
    return true; // keep the channel open for the async reply
  }
});

function downloadText(text, title) {
  const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");
  const filename = `${sanitizeFilename(title)}-${stamp}.txt`;
  const dataUrl = "data:text/plain;charset=utf-8," + encodeURIComponent(text);
  chrome.downloads.download({ url: dataUrl, filename, saveAs: false });
}

async function addToCollection(text) {
  const { snippets = [] } = await chrome.storage.local.get("snippets");
  snippets.push({ text, at: Date.now() });
  await chrome.storage.local.set({ snippets });
  chrome.action.setBadgeText({ text: String(snippets.length) });
  chrome.action.setBadgeBackgroundColor({ color: "#2563eb" });
}

function sanitizeFilename(name) {
  return (name || "selection")
    .replace(/[\/\\?%*:|"<>]/g, "")
    .replace(/\s+/g, "_")
    .slice(0, 50) || "selection";
}
