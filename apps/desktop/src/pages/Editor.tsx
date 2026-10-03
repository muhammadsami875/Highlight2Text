export default function Editor() {
  return (
    <section className="grid grid-cols-[200px_1fr_280px] h-full">
      <aside className="border-r border-neutral-800 bg-neutral-900 p-3 text-sm text-neutral-400">
        Pages
      </aside>
      <div className="flex items-center justify-center text-neutral-500 text-sm">
        Preview area — awaits Phase 3 image import.
      </div>
      <aside className="border-l border-neutral-800 bg-neutral-900 p-3 text-sm text-neutral-400">
        Tools
      </aside>
    </section>
  );
}
