import type { EnhancementParams, Preset } from "../types/bindings";

interface Props {
  params: EnhancementParams;
  onChange: (p: EnhancementParams) => void;
  disabled?: boolean;
}

const PRESETS: { key: Preset; label: string }[] = [
  { key: "Original", label: "Original" },
  { key: "Color", label: "Color" },
  { key: "Auto", label: "Auto" },
  { key: "Document", label: "Document" },
  { key: "BlackWhite", label: "B&W" },
  { key: "Grayscale", label: "Grayscale" },
  { key: "HighContrast", label: "High contrast" },
];

export default function EnhancementPanel({ params, onChange, disabled }: Props) {
  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-1">
        {PRESETS.map((p) => (
          <button
            key={p.key}
            disabled={disabled}
            onClick={() => onChange({ ...params, preset: p.key })}
            className={`text-xs rounded-md border py-1.5 ${
              params.preset === p.key
                ? "border-brand-500 text-brand-500"
                : "border-neutral-800 text-neutral-300 hover:border-neutral-600"
            }`}
          >
            {p.label}
          </button>
        ))}
      </div>
      <Slider label="Brightness" value={params.brightness} min={-100} max={100}
        onChange={(v) => onChange({ ...params, brightness: v })} disabled={disabled} />
      <Slider label="Contrast" value={params.contrast} min={-100} max={100}
        onChange={(v) => onChange({ ...params, contrast: v })} disabled={disabled} />
      <Slider label="Sharpness" value={params.sharpness} min={0} max={100}
        onChange={(v) => onChange({ ...params, sharpness: v })} disabled={disabled} />
      <Slider label="Shadow / background" value={params.shadow_remove} min={0} max={100}
        onChange={(v) => onChange({ ...params, shadow_remove: v })} disabled={disabled} />
    </div>
  );
}

function Slider(props: {
  label: string; value: number; min: number; max: number;
  onChange: (v: number) => void; disabled?: boolean;
}) {
  return (
    <label className="block">
      <div className="flex justify-between text-xs text-neutral-400">
        <span>{props.label}</span>
        <span className="font-mono text-neutral-500">{props.value}</span>
      </div>
      <input
        type="range"
        min={props.min}
        max={props.max}
        value={props.value}
        disabled={props.disabled}
        onChange={(e) => props.onChange(parseInt(e.target.value, 10))}
        className="w-full accent-brand-500"
      />
    </label>
  );
}
