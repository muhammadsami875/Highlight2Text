import { useRef, useState } from "react";
import type { ImportedPage } from "../types/bindings";
import { assetUrl } from "../services/ipc";

interface Props {
  pages: ImportedPage[];
  activeId: string | null;
  onSelect: (id: string) => void;
  onRemove: (id: string) => void;
  onReorder?: (from: number, to: number) => void;
}

export default function PageThumbnailStrip({ pages, activeId, onSelect, onRemove, onReorder }: Props) {
  const dragFrom = useRef<number | null>(null);
  const [dragOverIdx, setDragOverIdx] = useState<number | null>(null);

  if (pages.length === 0) {
    return (
      <div className="p-3 text-xs text-neutral-500">
        No pages yet. Drop an image here or use Import.
      </div>
    );
  }
  return (
    <ol className="p-2 space-y-2 overflow-y-auto">
      {pages.map((p, i) => (
        <li
          key={p.id}
          draggable={!!onReorder}
          onDragStart={() => { dragFrom.current = i; }}
          onDragOver={(e) => { e.preventDefault(); setDragOverIdx(i); }}
          onDragLeave={() => setDragOverIdx((v) => (v === i ? null : v))}
          onDrop={(e) => {
            e.preventDefault();
            const from = dragFrom.current;
            dragFrom.current = null;
            setDragOverIdx(null);
            if (from == null || !onReorder || from === i) return;
            onReorder(from, i);
          }}
          className={dragOverIdx === i ? "ring-2 ring-brand-500 rounded-md" : ""}
        >
          <button
            onClick={() => onSelect(p.id)}
            className={`group relative block w-full rounded-md border overflow-hidden ${
              activeId === p.id ? "border-brand-500" : "border-neutral-800 hover:border-neutral-700"
            }`}
          >
            <img
              src={assetUrl(p.thumbnail_path)}
              alt={`page ${i + 1}`}
              className="w-full h-24 object-contain bg-neutral-950"
              loading="lazy"
              draggable={false}
            />
            <span className="absolute left-1 top-1 text-[10px] bg-neutral-900/80 px-1 rounded">
              {i + 1}
            </span>
            <span
              role="button"
              aria-label="Remove page"
              onClick={(e) => { e.stopPropagation(); onRemove(p.id); }}
              className="absolute right-1 top-1 text-[10px] bg-neutral-900/80 px-1 rounded opacity-0 group-hover:opacity-100"
            >
              ✕
            </span>
          </button>
        </li>
      ))}
    </ol>
  );
}
