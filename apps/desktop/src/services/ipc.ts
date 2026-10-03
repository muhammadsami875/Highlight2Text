import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AppInfo, Corner, DetectedBoundary, EnhancedPage, EnhancementParams,
  ImportedPage, OcrOptions, OcrResult, PdfExportOptions, PdfExportPage,
  WarpedPage,
} from "../types/bindings";
import { save } from "@tauri-apps/plugin-dialog";

export type {
  AppInfo, DetectedBoundary, ImportedPage, Corner, WarpedPage,
  EnhancedPage, EnhancementParams, Preset, OcrOptions, OcrResult,
  OcrWord, PageMode, PdfExportOptions, PdfExportPage, PageSize,
  MarginSize, Quality,
} from "../types/bindings";

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function importImage(path: string): Promise<ImportedPage> {
  return invoke<ImportedPage>("import_image", { path });
}

export async function detectDocumentBoundary(path: string): Promise<DetectedBoundary> {
  return invoke<DetectedBoundary>("detect_document_boundary", { path });
}

export async function applyPerspective(
  path: string,
  corners: [Corner, Corner, Corner, Corner],
): Promise<WarpedPage> {
  return invoke<WarpedPage>("apply_perspective", { path, corners });
}

export async function applyEnhancement(
  path: string,
  params: EnhancementParams,
): Promise<EnhancedPage> {
  return invoke<EnhancedPage>("apply_enhancement", { path, params });
}

export async function runOcr(path: string, options: OcrOptions): Promise<OcrResult> {
  return invoke<OcrResult>("run_ocr", { path, options });
}

export async function importPdf(path: string): Promise<ImportedPage[]> {
  return invoke<ImportedPage[]>("import_pdf", { path });
}

export async function pickPdf(): Promise<string | null> {
  const picked = await open({ multiple: false, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  return typeof picked === "string" ? picked : null;
}

export async function exportPdf(
  pages: PdfExportPage[],
  outPath: string,
  options: PdfExportOptions,
): Promise<void> {
  return invoke("export_pdf", { pages, outPath, options });
}

export async function savePdfPath(): Promise<string | null> {
  const picked = await save({ defaultPath: "DocSnap.pdf", filters: [{ name: "PDF", extensions: ["pdf"] }] });
  return picked ?? null;
}

export async function exportText(outPath: string, bodies: string[]): Promise<void> {
  return invoke("export_text", { outPath, bodies });
}

export async function exportImage(
  sourcePath: string,
  outPath: string,
  kind: "Png" | "Jpeg" | "Webp" | "Tiff",
  quality: number,
): Promise<void> {
  return invoke("export_image", { sourcePath, outPath, kind, quality });
}

export async function savePathWith(ext: string, defaultName: string): Promise<string | null> {
  const picked = await save({
    defaultPath: `${defaultName}.${ext}`,
    filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
  });
  return picked ?? null;
}

export async function pickImages(): Promise<string[]> {
  const picked = await open({
    multiple: true,
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "tif", "tiff", "bmp", "webp"],
      },
    ],
  });
  if (!picked) return [];
  return Array.isArray(picked) ? picked : [picked];
}

/// Resolve a native file path to a URL the webview can render.
export function assetUrl(absPath: string): string {
  return convertFileSrc(absPath);
}
