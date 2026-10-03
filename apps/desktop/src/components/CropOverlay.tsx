import type { Corner } from "../types/bindings";

interface Props {
  /// Source-image pixel dimensions.
  srcWidth: number;
  srcHeight: number;
  /// Corners in SOURCE pixel coordinates (TL, TR, BR, BL).
  corners: [Corner, Corner, Corner, Corner];
  confidence: number;
  fallback: boolean;
}

/// Renders the detected boundary as an SVG overlay sized to its container.
/// Manual dragging lands in Phase 5; this is view-only.
export default function CropOverlay({
  srcWidth,
  srcHeight,
  corners,
  confidence,
  fallback,
}: Props) {
  const pts = corners.map((c) => `${c.x},${c.y}`).join(" ");
  const confPct = Math.round(confidence * 100);
  return (
    <svg
      viewBox={`0 0 ${srcWidth} ${srcHeight}`}
      preserveAspectRatio="xMidYMid meet"
      className="absolute inset-0 w-full h-full pointer-events-none"
    >
      <polygon
        points={pts}
        fill="rgba(43,108,255,0.12)"
        stroke={fallback ? "#f59e0b" : "#2b6cff"}
        strokeWidth={Math.max(2, Math.min(srcWidth, srcHeight) * 0.004)}
      />
      {corners.map((c, i) => (
        <circle
          key={i}
          cx={c.x}
          cy={c.y}
          r={Math.max(6, Math.min(srcWidth, srcHeight) * 0.01)}
          fill="#2b6cff"
          stroke="#ffffff"
          strokeWidth={Math.max(1, Math.min(srcWidth, srcHeight) * 0.002)}
        />
      ))}
      <text
        x={16}
        y={32}
        fill={fallback ? "#f59e0b" : "#2b6cff"}
        fontSize={Math.max(14, Math.min(srcWidth, srcHeight) * 0.02)}
        fontFamily="Inter, system-ui, sans-serif"
      >
        {fallback ? `fallback · ${confPct}% confidence` : `${confPct}% confidence`}
      </text>
    </svg>
  );
}
