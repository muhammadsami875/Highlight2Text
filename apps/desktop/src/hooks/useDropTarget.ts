import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";

/// Subscribes to Tauri's native drag-drop events. The web `drop` event does not
/// carry filesystem paths; we need the native stream.
export function useDropTarget(onDrop: (paths: string[]) => void) {
  const [over, setOver] = useState(false);

  useEffect(() => {
    const unlistenP = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        setOver(true);
      } else if (event.payload.type === "leave") {
        setOver(false);
      } else if (event.payload.type === "drop") {
        setOver(false);
        onDrop(event.payload.paths);
      }
    });
    return () => {
      unlistenP.then((u) => u());
    };
  }, [onDrop]);

  return over;
}
