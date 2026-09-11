// Shows a small floating "Save .txt" button whenever the user
// highlights text on the page. Clicking it downloads that text.

let btn = null;

document.addEventListener("mouseup", (e) => {
  if (btn && btn.contains(e.target)) return; // ignore clicks on our own button
  const text = (window.getSelection().toString() || "").trim();
  if (text.length > 0) {
    showButton();
  } else {
    removeButton();
  }
});

document.addEventListener("mousedown", (e) => {
  if (btn && !btn.contains(e.target)) removeButton();
});
document.addEventListener("scroll", removeButton, true);

function showButton() {
  removeButton();

  const sel = window.getSelection();
  if (!sel.rangeCount) return;
  const rect = sel.getRangeAt(0).getBoundingClientRect();

  btn = document.createElement("div");
  btn.textContent = "⬇ Save .txt";
  Object.assign(btn.style, {
    position: "fixed",
    top: Math.max(rect.top - 40, 6) + "px",
    left: Math.max(rect.left, 6) + "px",
    zIndex: "2147483647",
    background: "#2563eb",
    color: "#fff",
    font: "13px/1 system-ui, -apple-system, sans-serif",
    padding: "9px 11px",
    borderRadius: "7px",
    cursor: "pointer",
    boxShadow: "0 3px 10px rgba(0,0,0,.28)",
    userSelect: "none"
  });

  btn.addEventListener("click", () => {
    const text = (window.getSelection().toString() || "").trim();
    if (text) {
      chrome.runtime.sendMessage({
        type: "download",
        text,
        title: document.title
      });
    }
    removeButton();
  });

  document.body.appendChild(btn);
}

function removeButton() {
  if (btn && btn.parentNode) btn.parentNode.removeChild(btn);
  btn = null;
}
