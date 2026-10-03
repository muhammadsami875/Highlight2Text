import { useCallback, useRef, useState } from "react";
import type { Corner } from "../types/bindings";

type Quad = [Corner, Corner, Corner, Corner];

interface Props {
  srcWidth: number;
  srcHeight: number;
  corners: Quad;
  confidence: number;
  fallback: boolean;
  onChange: (q: Quad) => void;
  interactive?: boolean;
}

/// Interactive quadrilateral editor. Four draggable corners, each clamped
/// inside the source frame; drags that would make the polygon self-intersect
/// are rejected so Phase 6's perspective warp always receives a valid quad.
export default function CropEditor({
  srcWidth,
  srcHeight,
  corners,
  confidence,
  fallback,
  onChange,
  interactive = true,
}: Props) {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const [dragging, setDragging] = useState<number | null>(null);

  /// Convert a pointer event's client coords into source-pixel coords,
  /// respecting the xMidYMid-meet letterbox.
  const toSrcPoint = useCallback(
    (clientX: number, clientY: number) => {
      const svg = svgRef.current;
      if (!svg) return null;
      const rect = svg.getBoundingClientRect();
      const scale = Math.min(rect.width / srcWidth, rect.height / srcHeight);
      const padX = (rect.width - srcWidth * scale) / 2;
      const padY = (rect.height - srcHeight * scale) / 2;
      const x = (clientX - rect.left - padX) / scale;
      const y = (clientY - rect.top - padY) / scale;
      return {
        x: Math.max(0, Math.min(srcWidth, x)),
        y: Math.max(0, Math.min(srcHeight, y)),
      };
    },
    [srcWidth, srcHeight],
  );

  const handlePointerDown = (i: number) =>
    (e: React.PointerEvent<SVGCircleElement>) => {
      if (!interactive) return;
      e.preventDefault();
      e.currentTarget.setPointerCapture(e.pointerId);
      setDragging(i);
    };

  const handlePointerMove = (e: React.PointerEvent<SVGSVGElement>) => {
    if (dragging === null) return;
    const p = toSrcPoint(e.clientX, e.clientY);
    if (!p) return;
    const next: Quad = [...corners] as Quad;
    next[dragging] = p;
    if (!isSimpleQuad(next)) return;
    onChange(next);
  };

  const handlePointerUp = (e: React.PointerEvent<SVGSVGElement>) => {
    if (dragging === null) return;
    e.currentTarget.releasePointerCapture?.(e.pointerId);
    setDragging(null);
  };

  const pts = corners.map((c) => `${c.x},${c.y}`).join(" ");
  const confPct = Math.round(confidence * 100);
  const stroke = Math.max(2, Math.min(srcWidth, srcHeight) * 0.004);
  const handleR = Math.max(8, Math.min(srcWidth, srcHeight) * 0.012);
  const strokeColor = fallback ? "#f59e0b" : "#2b6cff";

  return (
    <svg
      ref={svgRef}
      viewBox={`0 0 ${srcWidth} ${srcHeight}`}
      preserveAspectRatio="xMidYMid meet"
      className={`absolute inset-0 w-full h-full ${
        interactive ? "touch-none" : "pointer-events-none"
      }`}
      onPointerMove={interactive ? handlePointerMove : undefined}
      onPointerUp={interactive ? handlePointerUp : undefined}
      onPointerCancel={interactive ? handlePointerUp : undefined}
    >
      <polygon
        points={pts}
        fill="rgba(43,108,255,0.10)"
        stroke={strokeColor}
        strokeWidth={stroke}
        strokeLinejoin="round"
        pointerEvents="none"
      />
      {corners.map((c, i) => (
        <g key={i}>
          <circle
            cx={c.x}
            cy={c.y}
            r={handleR * 2}
            fill="transparent"
            onPointerDown={handlePointerDown(i)}
            style={{ cursor: interactive ? "grab" : "default" }}
          />
          <circle
            cx={c.x}
            cy={c.y}
            r={handleR}
            fill={dragging === i ? "#ffffff" : strokeColor}
            stroke="#ffffff"
            strokeWidth={Math.max(1, Math.min(srcWidth, srcHeight) * 0.002)}
            pointerEvents="none"
          />
        </g>
      ))}
      <text
        x={16}
        y={Math.max(24, srcHeight * 0.03)}
        fill={strokeColor}
        fontSize={Math.max(14, Math.min(srcWidth, srcHeight) * 0.02)}
        fontFamily="Inter, system-ui, sans-serif"
        pointerEvents="none"
      >
        {fallback ? `fallback · ${confPct}%` : `${confPct}% confidence`}
      </text>
    </svg>
  );
}

/// A quad is "simple" if no two non-adjacent edges intersect.
function isSimpleQuad(q: Quad): boolean {
  const edges: Array<[Corner, Corner]> = [
    [q[0], q[1]],
    [q[1], q[2]],
    [q[2], q[3]],
    [q[3], q[0]],
  ];
  // Only edges (0,2) and (1,3) are non-adjacent pairs.
  return !segmentsIntersect(edges[0], edges[2]) && !segmentsIntersect(edges[1], edges[3]);
}

function segmentsIntersect(a: [Corner, Corner], b: [Corner, Corner]): boolean {
  const d1 = cross(sub(b[1], b[0]), sub(a[0], b[0]));
  const d2 = cross(sub(b[1], b[0]), sub(a[1], b[0]));
  const d3 = cross(sub(a[1], a[0]), sub(b[0], a[0]));
  const d4 = cross(sub(a[1], a[0]), sub(b[1], a[0]));
  if (((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) &&
      ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0))) return true;
  return false;
}

function sub(a: Corner, b: Corner): Corner { return { x: a.x - b.x, y: a.y - b.y }; }
function cross(a: Corner, b: Corner): number { return a.x * b.y - a.y * b.x; }
