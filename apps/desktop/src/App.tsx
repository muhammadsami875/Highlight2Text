import { NavLink, Route, Routes, Navigate } from "react-router-dom";
import Home from "./pages/Home";
import Scanner from "./pages/Scanner";
import Editor from "./pages/Editor";
import Ocr from "./pages/Ocr";
import Documents from "./pages/Documents";
import Settings from "./pages/Settings";

const navItems = [
  { to: "/home", label: "Home" },
  { to: "/scanner", label: "Scanner" },
  { to: "/documents", label: "Documents" },
  { to: "/ocr", label: "OCR" },
  { to: "/settings", label: "Settings" },
];

export default function App() {
  return (
    <div className="flex h-full">
      <aside className="w-56 bg-neutral-900 border-r border-neutral-800 flex flex-col">
        <div className="px-5 py-4 text-lg font-semibold tracking-tight">
          <span className="text-brand-500">Doc</span>Snap
        </div>
        <nav className="flex-1 px-2 space-y-1">
          {navItems.map((n) => (
            <NavLink
              key={n.to}
              to={n.to}
              className={({ isActive }) =>
                `block rounded-md px-3 py-2 text-sm transition ${
                  isActive
                    ? "bg-brand-500/15 text-brand-500"
                    : "text-neutral-300 hover:bg-neutral-800"
                }`
              }
            >
              {n.label}
            </NavLink>
          ))}
        </nav>
        <div className="px-4 py-3 text-xs text-neutral-500">v0.1.0 · offline</div>
      </aside>
      <main className="flex-1 overflow-auto">
        <Routes>
          <Route path="/" element={<Navigate to="/home" replace />} />
          <Route path="/home" element={<Home />} />
          <Route path="/scanner" element={<Scanner />} />
          <Route path="/editor/:projectId?" element={<Editor />} />
          <Route path="/ocr" element={<Ocr />} />
          <Route path="/documents" element={<Documents />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </main>
    </div>
  );
}
