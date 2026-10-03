import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import type { ImportedPage } from "../types/bindings";
import { useProject } from "../stores/projectStore";

export default function Scanner() {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const [devices, setDevices] = useState<MediaDeviceInfo[]>([]);
  const [deviceId, setDeviceId] = useState<string | null>(null);
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const addPages = useProject((s) => s.addPages);
  const nav = useNavigate();

  useEffect(() => () => stream?.getTracks().forEach((t) => t.stop()), [stream]);

  async function start() {
    setErr(null);
    try {
      const s = await navigator.mediaDevices.getUserMedia({
        video: deviceId ? { deviceId: { exact: deviceId } } : { facingMode: "environment" },
        audio: false,
      });
      setStream((prev) => { prev?.getTracks().forEach((t) => t.stop()); return s; });
      if (videoRef.current) videoRef.current.srcObject = s;
      const list = await navigator.mediaDevices.enumerateDevices();
      setDevices(list.filter((d) => d.kind === "videoinput"));
    } catch (e) {
      setErr(String((e as Error)?.message ?? e));
    }
  }

  async function capture() {
    const v = videoRef.current;
    if (!v) return;
    setBusy(true);
    try {
      const c = document.createElement("canvas");
      c.width = v.videoWidth;
      c.height = v.videoHeight;
      c.getContext("2d")!.drawImage(v, 0, 0);
      const dataUrl = c.toDataURL("image/png");
      const page = await invoke<ImportedPage>("save_capture", { dataUrl });
      addPages([page]);
    } catch (e) {
      setErr(String((e as Error)?.message ?? e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="p-6 h-full flex gap-6">
      <div className="flex-1 flex items-center justify-center bg-black rounded-lg overflow-hidden relative">
        {!stream && (
          <div className="absolute inset-0 flex items-center justify-center text-neutral-500 text-sm">
            Camera preview appears here after you grant permission.
          </div>
        )}
        <video ref={videoRef} autoPlay playsInline muted
          className="max-h-full max-w-full" />
      </div>
      <aside className="w-72 space-y-3 text-sm">
        <h2 className="text-base font-medium">Scanner</h2>
        <p className="text-xs text-neutral-500">
          Preview is live only after you press Start. Captures land as pages in
          the Editor — no uploads.
        </p>
        {devices.length > 0 && (
          <label className="block">
            <div className="text-xs text-neutral-400 mb-1">Camera</div>
            <select
              value={deviceId ?? ""}
              onChange={(e) => setDeviceId(e.target.value || null)}
              className="w-full bg-neutral-800 border border-neutral-700 rounded text-xs px-2 py-1"
            >
              {devices.map((d) => (
                <option key={d.deviceId} value={d.deviceId}>
                  {d.label || `Camera ${d.deviceId.slice(0, 6)}`}
                </option>
              ))}
            </select>
          </label>
        )}
        <div className="flex gap-2">
          <button onClick={start} className="flex-1 rounded-md bg-brand-500 hover:bg-brand-600 text-white py-2 text-sm">
            {stream ? "Restart" : "Start camera"}
          </button>
          <button
            disabled={!stream || busy}
            onClick={capture}
            className="flex-1 rounded-md border border-neutral-700 hover:border-brand-500/60 disabled:opacity-40 text-sm py-2"
          >
            {busy ? "Saving…" : "Capture"}
          </button>
        </div>
        <button
          onClick={() => nav("/editor")}
          className="w-full rounded-md border border-neutral-700 hover:border-brand-500/60 text-sm py-2"
        >
          Go to editor
        </button>
        {err && <div className="text-xs text-red-400 break-all">{err}</div>}
      </aside>
    </section>
  );
}
