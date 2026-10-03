import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { importImage, pickImages, pickPdf, importPdf } from "../services/ipc";
import { useProject } from "../stores/projectStore";

export function useShortcuts() {
  const nav = useNavigate();
  const addPages = useProject((s) => s.addPages);
  const activeId = useProject((s) => s.activeId);
  const removePage = useProject((s) => s.removePage);

  useEffect(() => {
    async function onKey(e: KeyboardEvent) {
      const mod = e.ctrlKey || e.metaKey;
      if (mod && !e.shiftKey && e.key.toLowerCase() === "o") {
        e.preventDefault();
        const paths = await pickImages();
        for (const p of paths) { try { addPages([await importImage(p)]); } catch {/*noop*/} }
        nav("/editor");
      } else if (mod && e.shiftKey && e.key.toLowerCase() === "o") {
        e.preventDefault();
        const p = await pickPdf();
        if (p) { try { addPages(await importPdf(p)); nav("/editor"); } catch {/*noop*/} }
      } else if (mod && !e.shiftKey && e.key.toLowerCase() === "e") {
        e.preventDefault(); nav("/export");
      } else if (mod && e.shiftKey && e.key.toLowerCase() === "e") {
        e.preventDefault(); nav("/ocr");
      } else if (mod && !e.shiftKey && e.key.toLowerCase() === "f") {
        e.preventDefault(); nav("/documents");
      } else if (e.key === "Delete" && activeId) {
        e.preventDefault();
        removePage(activeId);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [nav, addPages, activeId, removePage]);
}
