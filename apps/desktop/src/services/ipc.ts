import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AppInfo, Corner, DetectedBoundary, EnhancedPage, EnhancementParams,
  ImportedPage, WarpedPage,
} from "../types/bindings";

export type {
  AppInfo, DetectedBoundary, ImportedPage, Corner, WarpedPage,
  EnhancedPage, EnhancementParams, Preset,
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
