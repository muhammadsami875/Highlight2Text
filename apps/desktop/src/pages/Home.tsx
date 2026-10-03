import { useNavigate, Link } from "react-router-dom";
import { importImage, pickImages } from "../services/ipc";
import { useProject } from "../stores/projectStore";

export default function Home() {
  const nav = useNavigate();
  const addPages = useProject((s) => s.addPages);

  async function importFromPicker() {
    const paths = await pickImages();
    const results = [];
    for (const p of paths) {
      try { results.push(await importImage(p)); } catch { /* reported in editor */ }
    }
    if (results.length) {
      addPages(results);
      nav("/editor");
    }
  }

  return (
    <section className="p-8 max-w-5xl">
      <h1 className="text-2xl font-semibold mb-1">Welcome back</h1>
      <p className="text-neutral-400 mb-8">
        Scan, enhance, and extract text — fully on-device.
      </p>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <Tile title="Import images" desc="PNG, JPEG, TIFF, BMP, WebP." onClick={importFromPicker} />
        <Tile title="New scan" desc="Capture pages from your webcam (Phase 15)." disabled />
        <Tile title="Import PDF" desc="Open and process PDF pages (Phase 11)." disabled />
        <Link
          to="/documents"
          className="rounded-xl border border-neutral-800 bg-neutral-900 p-5 hover:border-brand-500/60 transition"
        >
          <div className="text-base font-medium">Documents</div>
          <div className="text-sm text-neutral-400 mt-1">Your saved projects and exports.</div>
        </Link>
      </div>
    </section>
  );
}

function Tile(props: { title: string; desc: string; onClick?: () => void; disabled?: boolean }) {
  return (
    <button
      onClick={props.onClick}
      disabled={props.disabled}
      className="text-left rounded-xl border border-neutral-800 bg-neutral-900 p-5 hover:border-brand-500/60 transition disabled:opacity-50 disabled:hover:border-neutral-800"
    >
      <div className="text-base font-medium">{props.title}</div>
      <div className="text-sm text-neutral-400 mt-1">{props.desc}</div>
    </button>
  );
}
