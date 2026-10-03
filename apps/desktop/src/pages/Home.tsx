import { Link } from "react-router-dom";

const tiles = [
  { to: "/scanner", title: "New Scan", desc: "Capture pages from your webcam." },
  { to: "/editor", title: "Import Image", desc: "PNG, JPEG, TIFF, BMP, WebP." },
  { to: "/editor", title: "Import PDF", desc: "Open and process PDF pages." },
  { to: "/documents", title: "Documents", desc: "Your saved projects and exports." },
];

export default function Home() {
  return (
    <section className="p-8 max-w-5xl">
      <h1 className="text-2xl font-semibold mb-1">Welcome back</h1>
      <p className="text-neutral-400 mb-8">
        Scan, enhance, and extract text — fully on-device.
      </p>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        {tiles.map((t) => (
          <Link
            key={t.title}
            to={t.to}
            className="rounded-xl border border-neutral-800 bg-neutral-900 p-5 hover:border-brand-500/60 transition"
          >
            <div className="text-base font-medium">{t.title}</div>
            <div className="text-sm text-neutral-400 mt-1">{t.desc}</div>
          </Link>
        ))}
      </div>
    </section>
  );
}
